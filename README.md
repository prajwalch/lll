[![CICD](https://github.com/PrajwalCH/lll/actions/workflows/CICD.yml/badge.svg?branch=main)](https://github.com/PrajwalCH/lll/actions/workflows/CICD.yml) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

# lll

lll is a simple local http server for serving files quickly.

## Building and running

Make sure you installed and setup the [rust toolchain](https://www.rust-lang.org/tools/install) first.

1. Clone this repo `git clone http://github.com/PrajwalCH/lll`
2. Jump to `lll` dir
3. Run `cargo build --release` to build it
4. Then run `cargo run -- [path]` to start it

By default, server runs on `2058` port, if you want to use different port run with `LLL_PORT` environment variable.

```bash
$ LLL_PORT=8080 cargo run -- [path]
```
