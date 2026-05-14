use std::{env, process};

use crate::{error::LoxResult, lox::Lox};

mod error;
mod lox;
mod model;
mod scanner;

fn main() -> LoxResult<()> {
    let args: Vec<String> = env::args().collect();
    let mut lox = Lox::new();
    if args.len() > 2 {
        println!("Usage: rlox [script]");
        process::exit(64);
    } else if args.len() == 2 {
        lox.run_file(&args[1])?;
    } else {
        lox.run_prompt()?;
    }
    Ok(())
}
