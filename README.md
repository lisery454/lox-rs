# lox-rs

Rust implementation of the Lox language.

ref: <https://craftinginterpreters.com/>

```shell
$env:RUST_BACKTRACE=1;
$env:RUSTFLAGS="-A warnings";
cargo run -p lox-interpreter -q -- test.lox
cargo run -p lox-compiler -q -- test.lox
```

## TODO

- function
- gc
- class
