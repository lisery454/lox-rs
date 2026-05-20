use std::{env, process};

use crate::{error::LoxResult, lox::Lox};

mod compiler;
mod error;
mod lox;
mod model;
mod scanner;
mod vm;

fn main() -> LoxResult<()> {
    let args: Vec<String> = env::args().collect();
    let mut lox = Lox::new();
    if args.len() > 2 {
        println!("Usage: rlox [script]");
        process::exit(64);
    } else if args.len() == 2 {
        if let Err(e) = lox.run_file(&args[1]) {
            eprintln!("{}", e);
            process::exit(1);
        }
    } else {
        if let Err(e) = lox.run_prompt() {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
    Ok(())
}
