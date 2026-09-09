# lox-rs

Rust implementation of the Lox language.

ref: <https://craftinginterpreters.com/>

```shell
$env:RUST_BACKTRACE=1;
cargo run -p lox-interpreter -- test.lox
cargo run -p lox-compiler -- test.lox
$env:RUSTFLAGS="-A warnings"; cargo run -p lox-compiler-new -q -- test.lox
```

## TODO

- function
- gc
- class
