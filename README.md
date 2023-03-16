# lll
llll is a simple local http server. I mainly built this tool to learn more about [Rust](https://rust-lang.org) but it is perfectly usable when you need to serve files locally without any configuration.

![lll_git](https://user-images.githubusercontent.com/42384293/224496530-19f6daa0-b66f-427d-aa2c-8183194e00db.png)

## Building and running
Before following below steps make sure you setup the [rust](https://www.rust-lang.org/) toolchain.

1. Clone this repo `git clone http://github.com/PrajwalCH/lll`
2. Jump to `lll` dir
3. Run `cargo build --release` to build it
4. Then run `cargo run -- [path]` to start it

By default server runs on `2058` port, if you want to use different port set the `LPORT` enviroment variable.

```bash
$ LPORT=8080 cargo run -- [path]
```

## License
 [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
