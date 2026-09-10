use std::{
    fs::{File, OpenOptions},
    io,
};

use anyhow::{Result, bail};
use std::io::Write;
use tabled::{builder::Builder, settings::Style};

use crate::model::{Chunk, Constant, Memory, ObjectKind, OpCode, Value};

pub struct VM<W: Write> {
    chunk: Option<Chunk>,
    ip: usize,

    log_file: Option<File>,
    memory: Memory,

    writer: W,
}

impl<W: Write> VM<W> {
    pub fn with_writer(writer: W) -> Self {
        Self {
            chunk: None,
            ip: 0,
            log_file: None,
            memory: Memory::new(),
            writer,
        }
    }
}

impl VM<io::Stdout> {
    pub fn new() -> Self {
        VM {
            chunk: None,
            ip: 0,
            log_file: None,
            memory: Memory::new(),
            writer: io::stdout(),
        }
    }
}

impl<W: Write> VM<W> {
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

    fn read_byte(&self) -> u8 {
        let byte = self.get_chunk().code[self.ip];
        byte
    }

    fn read_line(&self) -> usize {
        let line = self.get_chunk().lines[self.ip];
        line
    }

    fn get_offset(&mut self) -> usize {
        let o1 = (self.read_byte() as usize) << 8;
        self.ip += 1;
        let o2 = self.read_byte() as usize;
        self.ip += 1;
        let offset = o1 | o2;
        offset
    }

    fn read_constant(&mut self) -> Value {
        let index = self.read_byte() as usize;
        let constant = &self.get_chunk().constants[index];

        match constant {
            Constant::Number(n) => return Value::Number(*n),
            Constant::String(s) => {
                return self.string_to_value(s.clone());
            }
        }
    }

    fn string_to_value(&mut self, s: String) -> Value {
        let addr = self.memory.alloc_string(s);
        Value::Object(addr)
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
                    self.memory.stack_push(constant);
                }
                OpCode::Negate => {
                    let v = self.memory.stack_pop();
                    if let Value::Number(n) = v {
                        self.memory.stack_push(Value::Number(-n));
                    } else {
                        bail!("negate op must be used on a number, in line {}", line);
                    }
                }
                OpCode::Add => {
                    let b = self.memory.stack_pop();
                    let a = self.memory.stack_pop();
                    if let Value::Number(na) = a
                        && let Value::Number(nb) = b
                    {
                        self.memory.stack_push(Value::Number(na + nb));
                    } else if let Value::Object(addr_a) = a
                        && let Value::Object(addr_b) = b
                    {
                        let a = self.memory.get_obj(addr_a);
                        let b = self.memory.get_obj(addr_b);
                        if let Some(a) = a
                            && let Some(b) = b
                            && let ObjectKind::String(s_a) = a
                            && let ObjectKind::String(s_b) = b
                        {
                            let s = format!("{s_a}{s_b}");
                            let v = self.string_to_value(s);
                            self.memory.stack_push(v);
                        } else {
                            bail!("add op must be used on two same obj, in line {}", line);
                        }
                    } else {
                        bail!("invalid add op usage, in line {}", line);
                    }
                }
                OpCode::Subtract => {
                    let b = self.memory.stack_pop();
                    let a = self.memory.stack_pop();
                    if let Value::Number(na) = a
                        && let Value::Number(nb) = b
                    {
                        self.memory.stack_push(Value::Number(na - nb));
                    } else {
                        bail!("sub op must be used on two numbers, in line {}", line);
                    }
                }
                OpCode::Multiply => {
                    let b = self.memory.stack_pop();
                    let a = self.memory.stack_pop();
                    if let Value::Number(na) = a
                        && let Value::Number(nb) = b
                    {
                        self.memory.stack_push(Value::Number(na * nb));
                    } else {
                        bail!("multiply op must be used on two numbers, in line {}", line);
                    }
                }
                OpCode::Divide => {
                    let b = self.memory.stack_pop();
                    let a = self.memory.stack_pop();
                    if let Value::Number(na) = a
                        && let Value::Number(nb) = b
                    {
                        self.memory.stack_push(Value::Number(na / nb));
                    } else {
                        bail!("divide op must be used on two numbers, in line {}", line);
                    }
                }
                OpCode::Nil => {
                    self.memory.stack_push(Value::Nil);
                }
                OpCode::True => {
                    self.memory.stack_push(Value::Boolean(true));
                }
                OpCode::False => {
                    self.memory.stack_push(Value::Boolean(false));
                }
                OpCode::Not => {
                    let v = self.memory.stack_pop();
                    self.memory
                        .stack_push(Value::Boolean(!v.to_bool(&self.memory)));
                }
                OpCode::Equal => {
                    let b = self.memory.stack_pop();
                    let a = self.memory.stack_pop();
                    if let Value::Number(na) = a
                        && let Value::Number(nb) = b
                    {
                        self.memory.stack_push(Value::Boolean(na == nb));
                    } else if let Value::Boolean(na) = a
                        && let Value::Boolean(nb) = b
                    {
                        self.memory.stack_push(Value::Boolean(na == nb));
                    } else if let Value::Nil = a
                        && let Value::Nil = b
                    {
                        self.memory.stack_push(Value::Boolean(true));
                    } else if let Value::Object(addr_a) = a
                        && let Value::Object(addr_b) = b
                    {
                        let a = self.memory.get_obj(addr_a);
                        let b = self.memory.get_obj(addr_b);
                        if let Some(a) = a
                            && let Some(b) = b
                            && let ObjectKind::String(s_a) = a
                            && let ObjectKind::String(s_b) = b
                        {
                            self.memory.stack_push(Value::Boolean(s_a == s_b));
                        } else {
                            self.memory.stack_push(Value::Boolean(false));
                        }
                    } else {
                        self.memory.stack_push(Value::Boolean(false));
                    }
                }
                OpCode::Greater => {
                    let b = self.memory.stack_pop();
                    let a = self.memory.stack_pop();
                    if let Value::Number(na) = a
                        && let Value::Number(nb) = b
                    {
                        self.memory.stack_push(Value::Boolean(na > nb));
                    } else {
                        bail!("Operand must be a numbers, in line {}", line);
                    }
                }
                OpCode::Less => {
                    let b = self.memory.stack_pop();
                    let a = self.memory.stack_pop();
                    if let Value::Number(na) = a
                        && let Value::Number(nb) = b
                    {
                        self.memory.stack_push(Value::Boolean(na < nb));
                    } else {
                        bail!("Operand must be a numbers, in line {}", line);
                    }
                }
                OpCode::Print => {
                    let v = self.memory.stack_pop();
                    v.print(&self.memory, &mut self.writer)?;
                }
                OpCode::Pop => {
                    let _ = self.memory.stack_pop();
                }
                OpCode::DefineGlobal => {
                    // val is in stack, ip is on opcode DefineGlobal, name is on next pos of chunk.
                    let val = self.memory.stack_pop();
                    let name = self.read_constant();
                    self.ip += 1;

                    let Ok(name_str) = self.memory.get_string(name) else {
                        bail!("not find name obj when define global, in line {}", line);
                    };

                    self.memory.insert_global(&name_str, val);
                }
                OpCode::GetGlobal => {
                    let name = self.read_constant();
                    self.ip += 1;

                    let Ok(name_str) = self.memory.get_string(name) else {
                        bail!("not find name obj when get global, in line {}", line);
                    };

                    let Some(val) = self.memory.get_global(&name_str) else {
                        bail!("global {}  can't found, in line {}", name_str, line);
                    };

                    let val = val.clone();
                    self.memory.stack_push(val);
                }
                OpCode::SetGlobal => {
                    let new_val = self.memory.stack_peek().clone();
                    let name = self.read_constant();
                    self.ip += 1;
                    let Ok(name_str) = self.memory.get_string(name) else {
                        bail!("not find name obj when set global, in line {}", line);
                    };

                    let Ok(_) = self.memory.set_global(&name_str, new_val) else {
                        bail!("undefined variable {}, in line {}", name_str, line);
                    };
                }
                OpCode::GetLocal => {
                    let slot = self.read_byte() as usize;
                    self.ip += 1;
                    let Some(v) = self.memory.stack_get(slot).cloned() else {
                        bail!("not find local in {}, in line {}", slot, line);
                    };

                    self.memory.stack_push(v);
                }
                OpCode::SetLocal => {
                    let slot = self.read_byte() as usize;
                    self.ip += 1;
                    self.memory
                        .stack_set(slot, self.memory.stack_peek().clone());
                }
                OpCode::JumpIfFalse => {
                    let offset = self.get_offset();
                    if !self.memory.stack_peek().to_bool(&self.memory) {
                        self.ip += offset;
                    }
                }
                OpCode::Jump => {
                    let offset = self.get_offset();
                    self.ip += offset;
                }
                OpCode::RevJump => {
                    let offset = self.get_offset();
                    self.ip -= offset;
                }
            }
        }
    }
}

impl<W: Write> std::fmt::Display for VM<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = Builder::new();
        if let Some(chunk) = &self.chunk {
            builder.push_column([
                "chunk".to_string(),
                format!("{}", chunk.with_ip(self.ip as i32)),
            ]);
        }

        let stack_str = self
            .memory
            .stack
            .iter()
            .map(|ele| ele.to_string())
            .collect::<Vec<String>>()
            .join("\n");
        builder.push_column(["stack".to_string(), format!("{}", stack_str)]);

        let global_str = self
            .memory
            .heap
            .iter()
            .enumerate()
            .map(|(addr, ele)| match ele {
                Some(o) => format!("[{addr}]: {}", o.kind),
                None => format!("[{addr}]: <Nil>"),
            })
            .collect::<Vec<String>>()
            .join("\n");
        builder.push_column(["heap".to_string(), format!("{}", global_str)]);

        let global_str = self
            .memory
            .globals
            .iter()
            .map(|ele| format!("{}: {}", ele.0, ele.1))
            .collect::<Vec<String>>()
            .join("\n");
        builder.push_column(["globals".to_string(), format!("{}", global_str)]);

        let table = builder.build().with(Style::modern_rounded()).to_string();

        write!(f, "{}", table)
    }
}
