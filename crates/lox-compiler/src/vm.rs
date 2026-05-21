use std::{
    collections::HashMap,
    fmt,
    fs::{File, OpenOptions},
};

use crate::{
    error::LoxResult,
    model::{
        chunk::Chunk,
        opcode::OpCode,
        value::{Constant, Obj, Value},
    },
};
use std::io::Write;
use tabled::{builder::Builder, settings::Style};

const STACK_MAX: usize = 256;

pub struct VM {
    chunk: Option<Chunk>,
    ip: usize,
    stack: Vec<Value>,

    log_file: Option<File>,

    // TODO 目前仍然不会注册这些分配的内存对象，仍旧会内存泄漏
    allocated_objects: Vec<*mut Obj>,
    strings: HashMap<String, *mut Obj>,
    gloabls: HashMap<String, Value>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            chunk: None,
            ip: 0,
            stack: Vec::new(),
            allocated_objects: Vec::new(),
            strings: HashMap::new(),
            gloabls: HashMap::new(),
            log_file: None,
        }
    }

    pub fn with_log(mut self, path: &str) -> LoxResult<Self> {
        self.log_file = Some(
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)?,
        );

        Ok(self)
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

    fn read_byte(&mut self) -> u8 {
        let byte = self.get_chunk().code.borrow()[self.ip];
        self.ip += 1;
        byte
    }

    fn read_constant(&mut self) -> Value {
        let byte = self.read_byte();
        let c = self
            .get_chunk()
            .constants
            .borrow()
            .get(byte as usize)
            .unwrap()
            .clone();

        match c {
            Constant::Number(n) => return Value::Number(n),
            Constant::String(s) => {
                return self.string_to_value(s);
            }
        }
    }

    fn string_to_value(&mut self, s: String) -> Value {
        if let Some(p) = self.strings.get(&s) {
            Value::Obj(*p)
        } else {
            let raw_ptr = Box::into_raw(Box::new(Obj::String(s.clone())));
            self.strings.insert(s, raw_ptr);
            Value::Obj(raw_ptr)
        }
    }

    pub fn interpret(&mut self, chunk: Chunk) -> LoxResult<()> {
        self.chunk = Some(chunk);
        self.ip = 0;
        self.run()
    }

    pub fn run(&mut self) -> LoxResult<()> {
        loop {
            let log = format!("{}", self);
            if let Some(log_file) = &mut self.log_file {
                writeln!(log_file, "{}", log)?;
            }
            let instruction = self.read_byte();
            let line = self.get_chunk().get_line(self.ip);
            let code = OpCode::try_from(instruction)?;
            match code {
                OpCode::Constant => {
                    let constant = self.read_constant();
                    self.stack_push(constant);
                }
                OpCode::Return => {
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
                    } else if let Value::Obj(obj_a) = a
                        && let Value::Obj(obj_b) = b
                    {
                        unsafe {
                            let Obj::String(s_a) = &*obj_a;
                            let Obj::String(s_b) = &*obj_b;
                            let s = format!("{s_a}{s_b}");
                            let v = self.string_to_value(s);
                            self.stack_push(v);
                        }
                    } else {
                        return Err(crate::error::LoxError::RuntimeError {
                            message: "Operands must be two numbers or two strings.".into(),
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
                        Value::Obj(_) => self.stack_push(Value::Boolean(false)),
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
                    } else if let Value::Obj(obj_a) = a
                        && let Value::Obj(obj_b) = b
                    {
                        unsafe {
                            let Obj::String(_) = &*obj_a;
                            let Obj::String(_) = &*obj_b;
                            self.stack_push(Value::Boolean(obj_a == obj_b));
                        }
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
                OpCode::Print => {
                    let v = self.stack_pop();
                    println!("{}", v);
                }
                OpCode::Pop => {
                    let _ = self.stack_pop();
                }
                OpCode::DefineGlobal => {
                    // val is in stack, ip is on DefineGlobal, name is on next pos of chunk.
                    let val = self.stack_pop();
                    let name = self.read_constant();

                    if let Value::Obj(o) = name {
                        unsafe {
                            let Obj::String(name) = &*o;
                            self.gloabls.insert(name.clone(), val);
                        }
                    } else {
                        return Err(crate::error::LoxError::RuntimeError {
                            line,
                            message: "not find name when define global".into(),
                        });
                    }
                }
                OpCode::GetGlobal => {
                    let name = self.read_constant();
                    if let Value::Obj(o) = name {
                        unsafe {
                            let Obj::String(name) = &*o;
                            if let Some(val) = self.gloabls.get(name) {
                                self.stack_push(val.clone());
                            } else {
                                return Err(crate::error::LoxError::RuntimeError {
                                    line,
                                    message: format!("Undefined variable: {}", name),
                                });
                            }
                        }
                    } else {
                        return Err(crate::error::LoxError::RuntimeError {
                            line,
                            message: "not find name when get global".into(),
                        });
                    }
                }
                OpCode::SetGlobal => {
                    let new_val = self.stack_peek().clone();
                    let name = self.read_constant();
                    if let Value::Obj(o) = name {
                        unsafe {
                            let Obj::String(name) = &*o;

                            if let Some(val) = self.gloabls.get_mut(name) {
                                *val = new_val;
                            } else {
                                return Err(crate::error::LoxError::RuntimeError {
                                    line,
                                    message: format!("Undefined variable: {}", name),
                                });
                            }
                        }
                    } else {
                        return Err(crate::error::LoxError::RuntimeError {
                            line,
                            message: "not find name when set global".into(),
                        });
                    }
                }
                OpCode::GetLocal => {
                    let slot = self.read_byte() as usize;
                    let v = self.stack[slot].clone();
                    self.stack_push(v);
                }
                OpCode::SetLocal => {
                    let slot = self.read_byte() as usize;
                    self.stack[slot] = self.stack_peek().clone();
                }
            }
        }
    }
}

impl fmt::Display for VM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

        let global_str = self
            .gloabls
            .iter()
            .map(|ele| format!("{}: {}", ele.0, ele.1))
            .collect::<Vec<String>>()
            .join("\n");
        builder.push_column(["globals".to_string(), format!("{}", global_str)]);

        let table = builder.build().with(Style::modern_rounded()).to_string();

        write!(f, "{}", table)
    }
}

impl Drop for VM {
    fn drop(&mut self) {
        for obj in &self.allocated_objects {
            if !obj.is_null() {
                unsafe {
                    drop(Box::from_raw(*obj));
                }
            }
        }
    }
}
