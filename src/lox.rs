use anyhow::Result;
use std::{
    fs,
    io::{self, BufRead, Write},
};

use crate::{parser::Parser, scanner::Scanner};

pub struct Lox {}

impl Lox {
    pub fn new() -> Self {
        return Self {};
    }

    pub fn run_file(&mut self, path: &String) -> Result<()> {
        let code = fs::read_to_string(path)?;
        self.run(&code)?;
        Ok(())
    }

    pub fn run_prompt(&mut self) -> Result<()> {
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

            self.run(&line.to_string())?;
        }
        Ok(())
    }

    fn run(&mut self, code: &String) -> Result<()> {
        // println!("{:?}", code);

        let mut scanner = Scanner::new(code);
        let tokens = scanner.scan_tokens()?;

        // For now, just print the tokens.
        for token in tokens {
            println!("{}", token);
        }

        let mut parser = Parser::new(tokens);

        if let Some(expr) = parser.parse() {
            println!("{}", expr);

            println!("the result of expr is {}", expr.interpret()?);
        }

        Ok(())
    }
}
