[![CICD](https://github.com/PrajwalCH/lll/actions/workflows/CICD.yml/badge.svg?branch=main)](https://github.com/PrajwalCH/lll/actions/workflows/CICD.yml) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

# lll

lll is a simple local http server for serving files quickly.

## Install

Checkout the [releases](https://github.com/PrajwalCH/lll/releases) page for pre-built binaries.

## Usage

```bash
Usage: lll [options]

Options:
    -d, --dir PATH            Directory to serve (default: current)
    -p, --port PORT_NUM       Port to bind (default: 2058)
    -t, --expire-cache SECS   Set cache expiration time in seconds [default: 86400 (1 day)]
    -h, --help                Display help and exit
```

## Building and running

Make sure you installed and setup the [rust toolchain](https://www.rust-lang.org/tools/install) first.

1. Clone this repo `git clone http://github.com/PrajwalCH/lll`.
2. Jump to `lll` dir.
3. Run `cargo build --release` to build it.
4. Then run `cargo run -- [options]` or `./target/release/lll [options]` to start it.
5. To install run `cargo install --path .` or you can use pre-built binary.
