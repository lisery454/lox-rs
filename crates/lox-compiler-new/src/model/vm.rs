use std::fs::{File, OpenOptions};

use anyhow::{Ok, Result, bail};
use std::io::Write;
use tabled::{builder::Builder, settings::Style};

use crate::model::{Chunk, Constant, OpCode, Value};

pub struct VM {
    chunk: Option<Chunk>,
    ip: usize,
    stack: Vec<Value>,
    log_file: Option<File>,
}

impl VM {
    const STACK_MAX: usize = 256;

    pub fn new() -> Self {
        VM {
            chunk: None,
            ip: 0,
            stack: Vec::new(),
            log_file: None,
        }
    }

    fn get_chunk(&self) -> &Chunk {
        return match &self.chunk {
            Some(c) => c,
            None => panic!("chunk is none"),
        };
    }

    pub fn with_log(mut self, path: &str) -> Result<Self> {
        self.log_file = Some(
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)?,
        );

        Ok(self)
    }

    pub fn interpret(&mut self, chunk: Chunk) -> Result<()> {
        self.chunk = Some(chunk);
        self.ip = 0;
        self.run()
    }

    fn stack_push(&mut self, v: Value) {
        if self.stack.len() >= Self::STACK_MAX {
            panic!("stack over flow");
        }
        self.stack.push(v);
    }

    fn stack_peek(&self) -> &Value {
        let v = self.stack.last();
        if let Some(v) = v {
            return v;
        } else {
            panic!("stack is empty");
        }
    }

    fn stack_pop(&mut self) -> Value {
        let v = self.stack.pop();
        if let Some(v) = v {
            return v;
        } else {
            panic!("stack is empty");
        }
    }

    fn read_byte(&self) -> u8 {
        let byte = self.get_chunk().code[self.ip];
        byte
    }

    fn read_line(&self) -> usize {
        let line = self.get_chunk().lines[self.ip];
        line
    }

    fn read_constant(&self) -> Value {
        let index = self.read_byte();
        let c = self
            .get_chunk()
            .constants
            .get(index as usize)
            .unwrap()
            .clone();

        match c {
            Constant::Number(n) => return Value::Number(n),
        }
    }

    fn run(&mut self) -> Result<()> {
        loop {
            let log = format!("{}", self);
            if let Some(log_file) = &mut self.log_file {
                writeln!(log_file, "{}", log)?;
            }
            let instruction = self.read_byte();
            let line = self.read_line();
            self.ip += 1;
            let code = OpCode::try_from(instruction)?;
            match code {
                OpCode::Return => {
                    return Ok(());
                }
                OpCode::Constant => {
                    let constant = self.read_constant();
                    self.ip += 1;
                    self.stack_push(constant);
                }
                OpCode::Negate => {
                    let v = self.stack_pop();
                    if let Value::Number(n) = v {
                        self.stack_push(Value::Number(-n));
                    } else {
                        bail!("negate op must be used on a number, in line {}", line);
                    }
                }
                OpCode::Add => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();
                    if let Value::Number(na) = a
                        && let Value::Number(nb) = b
                    {
                        self.stack_push(Value::Number(na + nb));
                    } else {
                        bail!("add op must be used on two numbers, in line {}", line);
                    }
                }
                OpCode::Subtract => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();
                    if let Value::Number(na) = a
                        && let Value::Number(nb) = b
                    {
                        self.stack_push(Value::Number(na - nb));
                    } else {
                        bail!("sub op must be used on two numbers, in line {}", line);
                    }
                }
                OpCode::Multiply => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();
                    if let Value::Number(na) = a
                        && let Value::Number(nb) = b
                    {
                        self.stack_push(Value::Number(na * nb));
                    } else {
                        bail!("multiply op must be used on two numbers, in line {}", line);
                    }
                }
                OpCode::Divide => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();
                    if let Value::Number(na) = a
                        && let Value::Number(nb) = b
                    {
                        self.stack_push(Value::Number(na / nb));
                    } else {
                        bail!("divide op must be used on two numbers, in line {}", line);
                    }
                }
                OpCode::Nil => {
                    self.stack_push(Value::Nil);
                }
                OpCode::True => {
                    self.stack_push(Value::Boolean(true));
                }
                OpCode::False => {
                    self.stack_push(Value::Boolean(false));
                }
                OpCode::Not => {
                    let v = self.stack_pop();
                    self.stack_push(Value::Boolean(v.is_falsey()));
                }
                OpCode::Equal => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();
                    if let Value::Number(na) = a
                        && let Value::Number(nb) = b
                    {
                        self.stack_push(Value::Boolean(na == nb));
                    } else if let Value::Boolean(na) = a
                        && let Value::Boolean(nb) = b
                    {
                        self.stack_push(Value::Boolean(na == nb));
                    } else if let Value::Nil = a
                        && let Value::Nil = b
                    {
                        self.stack_push(Value::Boolean(true));
                    } else {
                        self.stack_push(Value::Boolean(false));
                    }
                }
                OpCode::Greater => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();
                    if let Value::Number(na) = a
                        && let Value::Number(nb) = b
                    {
                        self.stack_push(Value::Boolean(na > nb));
                    } else {
                        bail!("Operand must be a numbers, in line {}", line);
                    }
                }
                OpCode::Less => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();
                    if let Value::Number(na) = a
                        && let Value::Number(nb) = b
                    {
                        self.stack_push(Value::Boolean(na < nb));
                    } else {
                        bail!("Operand must be a numbers, in line {}", line);
                    }
                }
                OpCode::Print => {
                    let v = self.stack_pop();
                    println!("{}", v);
                }
                OpCode::Pop => {
                    let _ = self.stack_pop();
                }
                _ => {}
            }
        }
    }
}

impl std::fmt::Display for VM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = Builder::new();
        if let Some(chunk) = &self.chunk {
            builder.push_column([
                "chunk".to_string(),
                format!("{}", chunk.with_ip(self.ip as i32)),
            ]);
        }

        let stack_str = self
            .stack
            .iter()
            .map(|ele| ele.to_string())
            .collect::<Vec<String>>()
            .join("\n");
        builder.push_column(["stack".to_string(), format!("{}", stack_str)]);

        // let global_str = self
        //     .gloabls
        //     .iter()
        //     .map(|ele| format!("{}: {}", ele.0, ele.1))
        //     .collect::<Vec<String>>()
        //     .join("\n");
        // builder.push_column(["globals".to_string(), format!("{}", global_str)]);

        let table = builder.build().with(Style::modern_rounded()).to_string();

        write!(f, "{}", table)
    }
}
