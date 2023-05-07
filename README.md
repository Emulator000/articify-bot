# Articify - Bot

A Telegram AI creative bot based on Stable Diffusion

## Running the bot
The application uses `docker compose` in order to run all the needed services. You have to install last Docker CE with Docker compose in order to run it.

## Running and enter inside the container
If is the first time you can run `docker  compose build` (or `cargo make docker-build`) first, in order to make and build all containers.

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
You must compile the binary before running it, use the command `cargo make build` and then `cargo make run` in order to compile and execute it.

For the standard compile just run `cargo make release`.

You may need Rust installed to run all commands locally or just use the Docker container:
```
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
$ cargo install --force cargo-make
```

## Commands
Running:

* `./target/debug/bot` or `cargo make start-local` starts the local instance (WARNING: you must have MongoDB running locally)
* `./.scripts/run.sh start` or `cargo make start` for default: starts the dev instance
* `./.scripts/run.sh start prod` or `cargo make start-prod` for the production instance

*Append `-recreate` to each command in order to force the recreation of containers.*

Stopping:

* `./.scripts/run.sh stop` or `cargo make stop` to shutdown all containers
