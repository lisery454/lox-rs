use std::fmt;

use tabled::{builder::Builder, settings::Style};

use crate::{
    error::LoxResult,
    model::{chunk::Chunk, opcode::OpCode, value::Value},
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
            let line = self.get_chunk().get_line(self.ip);
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
                    if let Value::Number(n) = v {
                        self.stack_push(Value::Number(-n));
                    } else {
                        return Err(crate::error::LoxError::RuntimeError {
                            message: "Operand must be a number.".into(),
                            line: line,
                        });
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
                        return Err(crate::error::LoxError::RuntimeError {
                            message: "Operand must be a numbers.".into(),
                            line: line,
                        });
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
                        return Err(crate::error::LoxError::RuntimeError {
                            message: "Operand must be a numbers.".into(),
                            line: line,
                        });
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
                        return Err(crate::error::LoxError::RuntimeError {
                            message: "Operand must be a numbers.".into(),
                            line: line,
                        });
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
                        return Err(crate::error::LoxError::RuntimeError {
                            message: "Operand must be a numbers.".into(),
                            line: line,
                        });
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
                    match v {
                        Value::Boolean(b) => self.stack_push(Value::Boolean(!b)),
                        Value::Number(n) => {
                            if n == 0.0 {
                                self.stack_push(Value::Boolean(false));
                            } else {
                                self.stack_push(Value::Boolean(true));
                            }
                        }
                        Value::Nil => self.stack_push(Value::Boolean(true)),
                    }
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
                        return Err(crate::error::LoxError::RuntimeError {
                            message: "Operand must be a numbers.".into(),
                            line: line,
                        });
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
                        return Err(crate::error::LoxError::RuntimeError {
                            message: "Operand must be a numbers.".into(),
                            line: line,
                        });
                    }
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
