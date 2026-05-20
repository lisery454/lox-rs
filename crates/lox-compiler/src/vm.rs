use std::{collections::HashMap, fmt, rc::Rc};

use tabled::{builder::Builder, settings::Style};

use crate::{
    error::LoxResult,
    model::{
        chunk::Chunk,
        opcode::OpCode,
        value::{Constant, Obj, Value},
    },
};

const STACK_MAX: usize = 256;

pub struct VM {
    chunk: Option<Chunk>,
    ip: usize,
    stack: Vec<Value>,

    // TODO 目前仍然不会注册这些分配的内存对象，仍旧会内存泄漏
    allocated_objects: Vec<*mut Obj>,
    strings: HashMap<String, *mut Obj>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            chunk: None,
            ip: 0,
            stack: Vec::new(),
            allocated_objects: Vec::new(),
            strings: HashMap::new(),
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
                            let Obj::String(s_a) = &*obj_a;
                            let Obj::String(s_b) = &*obj_b;
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
