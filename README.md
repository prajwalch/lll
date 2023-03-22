# lll
llll is a simple local http server. I mainly built this tool to learn more about [Rust](https://rust-lang.org) but it is perfectly usable when you need to serve files locally without any configuration.

## Building and running
Make sure you installed and setup the [rust toolchain](https://www.rust-lang.org/tools/install) first.

1. Clone this repo `git clone http://github.com/PrajwalCH/lll`
2. Jump to `lll` dir
3. Run `cargo build --release` to build it
4. Then run `cargo run -- [path]` to start it

By default server runs on `2058` port, if you want to use different port run with `LPORT` enviroment variable.

```bash
$ LPORT=8080 cargo run -- [path]
```

## License
 [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
