# Articify - Bot

A Telegram AI creative bot based on Stable Diffusion

<table>
  <tr>
    <td><img src="./assets/showcase/img1.png" alt="Sample 1" width="360" height ="360"></td>
    <td><img src="./assets/showcase/img2.png" alt="Sample 2" width="360" height ="360"></td>
   </tr>
  <tr>
    <td><img src="./assets/showcase/img3.png" alt="Sample 3" width="360" height ="360"></td>
    <td><img src="./assets/showcase/img4.png" alt="Sample 4" width="360" height ="360"></td>
   </tr>
</table>

## Socials

- Telegram: <img align="center" src="./assets/showcase/telegram.png" alt="icon | Instagram" width="21px"/> <a href="https://t.me/articifyai/">articifyai</a>
- Instagram: <img align="center" src="./assets/showcase/instagram.png" alt="icon | Instagram" width="21px"/> <a href="https://www.instagram.com/articifyai/">articifyai</a>

## Running the bot

The application uses `docker compose` in order to run all the needed services. You have to install last Docker CE with
Docker compose in order to run it.

## Running and enter inside the container

If is the first time you can run `docker  compose build` (or `cargo make docker-build`) first, in order to make and
build all containers.

If all is already up and running, run these commands to get inside the container:

```
$ docker compose up -d
```

Or just run `cargo make up`

Then enter on the container with:

```
$ docker compose exec -ti bot bash
```

Or just run `cargo make server-shell`

## Compiling

You must compile the binary before running it, use the command `cargo make build` and then `cargo make run` in order to
compile and execute it.

For the standard compile just run `cargo make release`.

You may need Rust installed to run all commands locally or just use the Docker container:

```
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
$ cargo install --force cargo-make
```

## Commands

Running:

* `./target/debug/articify-bot` or `cargo make start-local` starts the local instance
* `./.scripts/run.sh start` or `cargo make start` for default: starts the dev instance
* `./.scripts/run.sh start prod` or `cargo make start-prod` for the production instance

*Append `-recreate` to each command in order to force the recreation of containers.*

Stopping:

* `./.scripts/run.sh stop` or `cargo make stop` to shutdown all containers

## Data folder

You must have all these updated Stable Diffusion weights in order to run the bot:

- `data/vocab.txt`
- `data/clip_v2.1.ot`
- `data/unet_v2.1.ot`
- `data/vae-new.ot`

You can grab copies from [here](https://huggingface.co/lmz/rust-stable-diffusion-v2-1/tree/main/weights).
