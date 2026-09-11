use std::{
    env, fs,
    io::{self, BufRead, Write},
    process,
};

use anyhow::Result;

mod compiler;
mod model;
mod scanner;
mod test;
mod vm;

pub use compiler::*;
pub use scanner::*;
pub use vm::*;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        println!("Usage: rlox [script]");
        process::exit(64);
    } else if args.len() == 2 {
        if let Err(e) = run_file(&args[1]) {
            eprintln!("{}", e);
            process::exit(1);
        }
    } else {
        if let Err(e) = run_prompt() {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
    Ok(())
}

pub fn run_prompt() -> Result<()> {
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut line = String::new();
        let bytes_read = handle.read_line(&mut line)?;

        if bytes_read == 0 {
            println!();
            break;
        }

        let content = line.trim();
        if content.is_empty() {
            continue;
        }

        let code = line.to_string();
        run(&code)?;
    }
    Ok(())
}

fn run_file(path: &String) -> Result<()> {
    let code = fs::read_to_string(path)?;
    run(&code)?;
    Ok(())
}

fn run(code: &String) -> Result<()> {
    let function = Compiler::new(code).compile()?;
    VM::new().with_log("log.txt")?.interpret(function)?;
    Ok(())
}
