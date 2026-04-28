mod error;
mod lox;
mod scanner;
mod model;
mod parser;

use std::{env, process};
use anyhow::{Ok, Result};
use crate::lox::Lox;

fn main() -> Result<()> {
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
