# lll
lll is a simple local http server. I mainly build this tool to learn more about [rust](https://www.rust-lang.org) but it is completely useable when you need to quickly serve files locally without any configuration. 

![lll_git](https://user-images.githubusercontent.com/42384293/224496530-19f6daa0-b66f-427d-aa2c-8183194e00db.png)

## Building and running
### Dependencies
- [rust v1.68.0+](https://www.rust-lang.org/)

### For building and running manually
```bash
$ cargo build --release
$ ./target/release/lll [path]
```
### For building and running
```bash
$ cargo run --release -- [path]
```

## License
 [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
