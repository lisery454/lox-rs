use std::fmt;

use tabled::{builder::Builder, settings::Style};

use crate::{
    error::LoxResult,
    model::{
        chunk::{Chunk, OpCode},
        value::Value,
    },
};

const STACK_MAX: usize = 256;

pub struct VM {
    chunk: Option<Chunk>,
    ip: usize,
    stack: Vec<Value>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            chunk: None,
            ip: 0,
            stack: Vec::new(),
        }
    }

    fn get_chunk(&self) -> &Chunk {
        return match &self.chunk {
            Some(c) => c,
            None => panic!("chunk is none"),
        };
    }

    fn stack_push(&mut self, v: Value) {
        if self.stack.len() >= STACK_MAX {
            panic!("stack over flow");
        }
        self.stack.push(v);
    }

    fn stack_pop(&mut self) -> Value {
        let v = self.stack.pop();
        if let Some(v) = v {
            return v;
        } else {
            panic!("stack is empty");
        }
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.get_chunk().code.borrow()[self.ip];
        self.ip += 1;
        byte
    }

    fn read_constant(&mut self) -> Value {
        let byte = self.read_byte();
        let i = self.get_chunk().constants.borrow().values[byte as usize];
        i
    }

    pub fn interpret(&mut self, chunk: Chunk) -> LoxResult<()> {
        self.chunk = Some(chunk);
        self.ip = 0;
        self.run()
    }

    pub fn run(&mut self) -> LoxResult<()> {
        loop {
            // println!("{}", &self);
            let instruction = self.read_byte();
            let code = OpCode::try_from(instruction)?;
            match code {
                OpCode::Constant => {
                    let constant = self.read_constant();
                    self.stack_push(constant);
                }
                OpCode::Return => {
                    let v = self.stack_pop();
                    println!("value: {}", v);
                    return Ok(());
                }
                OpCode::Negate => {
                    let v = self.stack_pop();
                    self.stack_push(-v);
                }
                OpCode::Add => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();
                    self.stack_push(a + b);
                }
                OpCode::Subtract => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();
                    self.stack_push(a - b);
                }
                OpCode::Multiply => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();
                    self.stack_push(a * b);
                }
                OpCode::Divide => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();
                    self.stack_push(a / b);
                }
            }
        }
    }
}

impl fmt::Display for VM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = Builder::new();
        if let Some(chunk) = &self.chunk {
            builder.push_column(["chunk".to_string(), format!("{}", chunk)]);
        }

        builder.push_column(["ip".to_string(), format!("{}", self.ip)]);

        let mut stack_str = String::new();
        for ele in &self.stack {
            stack_str.push_str(&format!("{}\n", ele));
        }
        builder.push_column(["stack".to_string(), format!("{}", stack_str)]);

        let table = builder.build().with(Style::modern_rounded()).to_string();

        write!(f, "{}", table)
    }
}
