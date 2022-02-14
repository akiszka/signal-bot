# Signal Bot &middot; [![ci](https://github.com/akiszka/signal-bot/actions/workflows/build_container.yml/badge.svg)](https://github.com/akiszka/signal-bot/actions/workflows/build_container.yml) ![GitHub license](https://img.shields.io/badge/license-MIT-blue.svg)

This is a dockerized bot for the [Signal Messenger](https://signal.org/en/). It can be used to log in to Signal and send messages over the Internet.

# Features

* containerized
* written in Rust
* supports reproducible builds with [Nix](https://nixos.org/)

# Usage

Download the container from GitHub Container Registry:

```sh
docker pull ghcr.io/akiszka/signal-bot:latest
```

Then, start the container:

```sh
docker run --rm -itp 52340:8000 --tmpfs /tmp:exec ghcr.io/akiszka/signal-bot
```
[[[note: add /data mount, explain why /tmp is exec, TODO: start signal-cli deamon inside rust]]]

# Building

If you want to build signal-bot for development, using cargo will suffice. For example, this will build the project from source and run it:

```sh
cargo run
```

> **NOTE: you will need to have [signal-cli](https://github.com/AsamK/signal-cli) installed to run the project with cargo. However, building the container image with nix-build will download signal-cli for you.**

If you want to build the container image yourself, you will need to have Nix installed. Then, running the following command will read the instructions at default.nix, download all the dependencies, build the container image and place it under `./result`.

```sh
nix-build
```

After that, you can load the image into Docker:

```sh
docker load -i result
```

# Roadmap

- [ ] add tests
- [ ] add access control (possibly with JWT)
- [ ] add webhook support for GitHub, Expo, etc.