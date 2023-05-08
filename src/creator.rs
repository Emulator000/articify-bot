use anyhow::Context;
use anyhow::Result;
use diffusers::transformers::clip::ClipTextTransformer;
use diffusers::{pipelines::stable_diffusion, transformers::clip::Tokenizer};
use itertools::Itertools;
use std::fs;
use std::future::Future;
use std::pin::Pin;
use tch::{nn::Module, Cuda, Device, Kind, Tensor};
use uuid::Uuid;

static DEBUG: bool = false;

// Settings
static MAX_WIDTH: u32 = 512;
static MAX_HEIGHT: u32 = 768;
static NEGATIVE_PROMPT: &str = "";

// Stable Diffusion - Values
static VAE_CONSTANT: f64 = 0.18215;
static GUIDANCE_SCALE: f64 = 7.5;

// Stable Diffusion - Configuration
pub static SEED: i64 = 0;
static STEPS: usize = 50;
static SKIP_STEPS: bool = false;
static DEVICE: Device = Device::Cuda(0);
static VOCAB_PATH: &'static str = "data/vocab.txt";
static CLIP_PATH: &'static str = "data/clip_v2.1.ot";
static UNET_PATH: &'static str = "data/unet_v2.1.ot";
static VAE_PATH: &'static str = "data/vae-new.ot";

fn get_device() -> Device {
    if let Device::Cuda(_) = DEVICE {
        if Cuda::is_available() {
            DEVICE
        } else {
            Device::Cpu
        }
    } else {
        DEVICE
    }
}

fn get_steps() -> usize {
    if Cuda::is_available() {
        STEPS
    } else {
        STEPS / 2
    }
}

fn get_max_width() -> u32 {
    if Cuda::is_available() {
        MAX_WIDTH
    } else {
        512
    }
}

fn get_max_height() -> u32 {
    if Cuda::is_available() {
        MAX_HEIGHT
    } else {
        512
    }
}

fn generate_tokens<S: AsRef<str>>(
    prompt: S,
    clip_device: Device,
    tokenizer: &Tokenizer,
) -> Result<Vec<Tensor>> {
    prompt
        .as_ref()
        .split(';')
        .try_fold(Vec::new(), |mut acc_tokens, prompt| {
            let tokens = tokenizer.encode(prompt.trim())?;
            let tokens = tokens.into_iter().map(|x| x as i64).collect::<Vec<_>>();

            acc_tokens.push(Tensor::of_slice(&tokens).view((1, -1)).to(clip_device));

            Ok::<_, anyhow::Error>(acc_tokens)
        })
}

fn generate_embeddings(tokens: Vec<Tensor>, text_model: &ClipTextTransformer) -> Result<Tensor> {
    let mut embeddings = None;
    tokens.iter().for_each(|token| {
        embeddings = Some(text_model.forward(token));
    });

    embeddings.context("Could get embeddings")
}

#[allow(clippy::too_many_arguments)]
pub async fn generate<S: AsRef<str>>(
    prompt: S,
    seed: i64,
    watermark: bool,
    callback: impl Fn(String) -> Pin<Box<dyn Future<Output = ()>>>,
) -> Result<Vec<u8>> {
    if let Device::Cuda(_) = DEVICE {
        tch::maybe_init_cuda();

        log::info!("Cuda available: {}", Cuda::is_available());
        log::info!("Cudnn available: {}", Cuda::cudnn_is_available());
    }

    let vocab_file = VOCAB_PATH.to_string();
    let clip_weights = CLIP_PATH;
    let unet_weights = UNET_PATH;
    let vae_weights = VAE_PATH;

    let sd_config = stable_diffusion::StableDiffusionConfig::v2_1(
        None,
        Some(get_max_height() as i64),
        Some(get_max_width() as i64),
    );

    let cpu = if let Device::Cuda(_) = get_device() {
        vec![]
    } else {
        vec!["all".to_string()]
    };

    log::info!("Setting up devices...");
    let device_setup = diffusers::utils::DeviceSetup::new(cpu);
    let clip_device = device_setup.get("clip");
    let vae_device = device_setup.get("vae");
    let unet_device = device_setup.get("unet");
    let scheduler = sd_config.build_scheduler(get_steps());

    log::info!("Building Tokenizer...");
    let tokenizer = Tokenizer::create(&vocab_file, &sd_config.clip)?;
    let tokens = generate_tokens(prompt, clip_device, &tokenizer)?;
    let uncond_tokens = generate_tokens(NEGATIVE_PROMPT, clip_device, &tokenizer)?;

    let no_grad_guard = tch::no_grad_guard();

    if seed != 0 {
        tch::manual_seed(seed);
    }

    log::info!("Building the Clip transformer...");
    let text_model = sd_config.build_clip_transformer(clip_weights, clip_device)?;
    let text_embeddings = generate_embeddings(tokens, &text_model)?;
    let uncond_text_embeddings = generate_embeddings(uncond_tokens, &text_model)?;
    let embeddings = Tensor::cat(&[uncond_text_embeddings, text_embeddings], 0).to(unet_device);

    log::info!("Building the Autoencoder...");
    let vae = sd_config.build_vae(vae_weights, vae_device)?;

    log::info!("Building the unet...");
    let unet = sd_config.build_unet(unet_weights, unet_device, 4)?;

    log::info!("Preparing latents...");
    let mut latents = Tensor::randn(
        &[1, 4, sd_config.height / 8, sd_config.width / 8],
        (Kind::Float, unet_device),
    );

    latents *= scheduler.init_noise_sigma();

    if !SKIP_STEPS {
        log::info!("Generating the image, please wait...");
        callback("⚙️ Generating the image, please wait...".into()).await;

        for (timestep_index, &timestep) in scheduler.timesteps().iter().enumerate() {
            log::info!("Timestep {}/{}", timestep_index + 1, get_steps());

            // expand the latents
            let latent_model_input = Tensor::cat(&[&latents, &latents], 0);

            // concat latents, mask, masked_image_latents in the channel dimension
            let latent_model_input = scheduler.scale_model_input(latent_model_input, timestep);

            // predict the noise residual
            let noise_pred = unet.forward(&latent_model_input, timestep as f64, &embeddings);

            // perform guidance
            let (noise_pred_uncond, noise_pred_text) = noise_pred
                .chunk(2, 0)
                .into_iter()
                .collect_tuple()
                .context("Couldn't collect tuples")?;
            let noise_pred =
                &noise_pred_uncond + (noise_pred_text - &noise_pred_uncond) * GUIDANCE_SCALE;

            // compute the previous noisy sample x_t -> x_t-1
            latents = scheduler.step(&noise_pred, timestep, &latents)
        }
    }

    log::info!("Generating the final image...");
    callback("🖼 Generating the final photo...".into()).await;
    latents = latents.to(vae_device);
    latents = vae.decode(&(&latents / VAE_CONSTANT));
    latents = (latents / 2 + 0.5).clamp(0., 1.).to_device(get_device());
    latents = (latents * 255.).to_kind(Kind::Uint8);

    let image_path = format!("temp/{}.png", Uuid::new_v4());

    log::info!("Saving image...");
    tch::vision::image::save(&latents, image_path.as_str())?;

    drop(no_grad_guard);

    let mut final_image = photon_rs::native::open_image(image_path.as_str())?;

    if !DEBUG {
        fs::remove_file(image_path.as_str())?;
    }

    if watermark {
        // watermark image
        log::info!("Applying watermark...");
        callback("⏳ Applying watermark...".into()).await;
        let water_mark = photon_rs::transform::resize(
            &photon_rs::native::open_image("assets/watermark.png")?,
            get_max_width(),
            get_max_height(),
            photon_rs::transform::SamplingFilter::Lanczos3,
        );
        photon_rs::multiple::watermark(&mut final_image, &water_mark, 0, 0);
    }

    log::info!("Image created successfully.");
    callback("✅ Image created successfully.".into()).await;

    Ok(final_image.get_bytes())
}
