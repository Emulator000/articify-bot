use crate::creator;
use crate::creator::GUIDANCE_SCALE;
use anyhow::Result;
use sscanf::sscanf;
use teloxide_core::adaptors::DefaultParseMode;
use teloxide_core::prelude::*;
use teloxide_core::types::{InputFile, Message, Update, UpdateKind};

pub async fn handle_update(bot: DefaultParseMode<Bot>, update: &Update) -> Result<()> {
    match &update.kind {
        UpdateKind::Message(message) => {
            if handle_commands(bot, update, message).await? {
                return Ok(());
            }
        }
        UpdateKind::EditedMessage(_) => {}
        UpdateKind::ChannelPost(_) => {}
        UpdateKind::EditedChannelPost(_) => {}
        UpdateKind::InlineQuery(_) => {}
        UpdateKind::ChosenInlineResult(_) => {}
        UpdateKind::CallbackQuery(_) => {}
        UpdateKind::ShippingQuery(_) => {}
        UpdateKind::PreCheckoutQuery(_) => {}
        UpdateKind::Poll(_) => {}
        UpdateKind::PollAnswer(_) => {}
        UpdateKind::MyChatMember(_) => {}
        UpdateKind::ChatMember(_) => {}
        UpdateKind::ChatJoinRequest(_) => {}
        UpdateKind::Error(_) => {}
    }

    Ok(())
}

async fn handle_commands(
    bot: DefaultParseMode<Bot>,
    update: &Update,
    message: &Message,
) -> Result<bool> {
    match update.user() {
        Some(user) => {
            log::debug!("New message received from {user:#?} with message {message:#?}.");

            if let Some(text) = message.text() {
                if let Ok((seed, guidance_scale, prompt)) =
                    sscanf!(text, "/generate {i64} {f64} {str}")
                {
                    let _ = generate_picture(bot, update, message, prompt, seed, guidance_scale)
                        .await?;

                    Ok(true)
                } else if let Ok((seed, prompt)) = sscanf!(text, "/generate {i64} {str}") {
                    let _ = generate_picture(bot, update, message, prompt, seed, GUIDANCE_SCALE)
                        .await?;

                    Ok(true)
                } else if let Ok((guidance_scale, prompt)) = sscanf!(text, "/generate {f64} {str}")
                {
                    let _ = generate_picture(
                        bot,
                        update,
                        message,
                        prompt,
                        creator::SEED,
                        guidance_scale,
                    )
                    .await?;

                    Ok(true)
                } else if let Ok(prompt) = sscanf!(text, "/generate {str}") {
                    let _ = generate_picture(
                        bot,
                        update,
                        message,
                        prompt,
                        creator::SEED,
                        GUIDANCE_SCALE,
                    )
                    .await?;

                    Ok(true)
                } else if text.eq("/start") {
                    let _ = bot
                        .send_message(message.chat.id, format!(r#"🤖 Hi! Please hint <code>/generate YOUR PROMPT</code> in order to create a new image.

You can also generate images with a predicted seed with: <code>/generate SEED-NUMBER YOUR PROMPT</code>.

Happy generation!"#))
                        .await?;

                    Ok(true)
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        }
        None => Ok(false),
    }
}

async fn generate_picture<S: AsRef<str>>(
    bot: DefaultParseMode<Bot>,
    update: &Update,
    message: &Message,
    prompt: S,
    seed: i64,
    guidance_scale: f64,
) -> Result<()> {
    let chat_id = message.chat.id;

    log::debug!(
        "New generation requests received from {:#?}.",
        update.user()
    );

    let edit_message = bot
        .send_message(chat_id, "⚙️ Processing your request...")
        .reply_to_message_id(message.id)
        .await?;

    match creator::generate(prompt, seed, guidance_scale, true, |text| {
        let bot = bot.clone();
        Box::pin(async move {
            let _ = bot.edit_message_text(chat_id, edit_message.id, text).await;
        })
    })
    .await
    {
        Ok(image) => {
            let _ = bot
                .send_photo(chat_id, InputFile::memory(image))
                .reply_to_message_id(message.id)
                .await?;
        }
        Err(e) => {
            log::error!("Image processing failed: {e:#?}");

            let _ = bot
                .send_message(chat_id, format!("❌ Photo processing failed: {e:#?}"))
                .reply_to_message_id(message.id)
                .await?;
        }
    }

    Ok(())
}
