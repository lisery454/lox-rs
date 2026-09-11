use std::{
    fs::{File, OpenOptions},
    io,
};

use anyhow::{Result, bail};
use std::io::Write;
use tabled::{builder::Builder, settings::Style};

use crate::model::{Chunk, Constant, Function, Memory, ObjectKind, OpCode, Value};

pub struct CallFrame {
    pub chunk: Chunk,
    pub ip: usize,
    pub stack_base: usize,
}

pub struct VM<W: Write> {
    frames: Vec<CallFrame>,

    log_file: Option<File>,
    memory: Memory,

    writer: W,
}

impl<W: Write> VM<W> {
    pub fn with_writer(writer: W) -> Self {
        Self {
            frames: Vec::new(),
            log_file: None,
            memory: Memory::new(),
            writer,
        }
    }
}

impl VM<io::Stdout> {
    pub fn new() -> Self {
        VM {
            frames: Vec::new(),
            log_file: None,
            memory: Memory::new(),
            writer: io::stdout(),
        }
    }
}

impl<W: Write> VM<W> {
    fn current_frame(&self) -> &CallFrame {
        self.frames.last().expect("call stack is empty")
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

    pub fn interpret(&mut self, function: Function) -> Result<()> {
        // 克隆 chunk 供 frame 使用，函数对象本身放到堆上
        let chunk = function.chunk.clone();
        let addr = self.memory.alloc(ObjectKind::Function(function));
        self.memory.stack_push(Value::Object(addr));

        let frame = CallFrame {
            chunk,
            ip: 0,
            stack_base: 0,
        };
        self.frames.push(frame);
        self.run()
    }

    fn read_byte(&self) -> u8 {
        let frame = self.current_frame();
        frame.chunk.code[frame.ip]
    }

    fn read_line(&self) -> usize {
        let frame = self.current_frame();
        frame.chunk.lines[frame.ip]
    }

    fn advance_ip(&mut self) {
        self.frames.last_mut().expect("call stack is empty").ip += 1;
    }

    fn get_offset(&mut self) -> usize {
        let o1 = (self.read_byte() as usize) << 8;
        self.advance_ip();
        let o2 = self.read_byte() as usize;
        self.advance_ip();
        let offset = o1 | o2;
        offset
    }

    fn read_constant(&mut self) -> Value {
        let index = self.read_byte() as usize;
        let constant = self.current_frame().chunk.constants[index].clone();

        match constant {
            Constant::Number(n) => Value::Number(n),
            Constant::String(s) => self.string_to_value(s),
            Constant::Function(func) => {
                let addr = self.memory.alloc(ObjectKind::Function(func));
                Value::Object(addr)
            }
        }
    }

    fn string_to_value(&mut self, s: String) -> Value {
        let addr = self.memory.alloc_string(s);
        Value::Object(addr)
    }

    fn call_value(&mut self, arg_count: usize) -> Result<()> {
        let callee_idx = self.memory.stack.len().saturating_sub(arg_count + 1);
        let Some(callee) = self.memory.stack_get(callee_idx).cloned() else {
            bail!("stack underflow when calling");
        };

        let Value::Object(addr) = callee else {
            bail!("can only call functions and classes");
        };

        // 取出函数信息后立即结束对 memory 的借用
        let (chunk, arity) = {
            let Some(ObjectKind::Function(func)) = self.memory.get_obj(addr) else {
                bail!("can only call functions and classes");
            };
            (func.chunk.clone(), func.arity)
        };

        if arity != arg_count {
            bail!("expected {} arguments but got {}", arity, arg_count);
        }

        self.frames.push(CallFrame {
            chunk,
            ip: 0,
            stack_base: callee_idx,
        });
        Ok(())
    }

    fn run(&mut self) -> Result<()> {
        loop {
            let log = format!("{}", self);
            if let Some(log_file) = &mut self.log_file {
                writeln!(log_file, "{}", log)?;
            }
            let instruction = self.read_byte();
            let line = self.read_line();
            self.advance_ip();
            let code = OpCode::try_from(instruction)?;
            match code {
                OpCode::Return => {
                    let result = self.memory.stack_pop();
                    let frame = self.frames.pop().expect("call stack is empty");
                    if self.frames.is_empty() {
                        // 脚本结束：弹出脚本函数对象
                        self.memory.stack.pop();
                        return Ok(());
                    }
                    // 恢复到 callee 位置并压入返回值
                    self.memory.stack.truncate(frame.stack_base);
                    self.memory.stack_push(result);
                }
                OpCode::Constant => {
                    let constant = self.read_constant();
                    self.advance_ip();
                    self.memory.stack_push(constant);
                }
                OpCode::Call => {
                    let arg_count = self.read_byte() as usize;
                    self.advance_ip();
                    self.call_value(arg_count)?;
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
                    self.advance_ip();

                    let Ok(name_str) = self.memory.get_string(name) else {
                        bail!("not find name obj when define global, in line {}", line);
                    };

                    self.memory.insert_global(&name_str, val);
                }
                OpCode::GetGlobal => {
                    let name = self.read_constant();
                    self.advance_ip();

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
                    self.advance_ip();
                    let Ok(name_str) = self.memory.get_string(name) else {
                        bail!("not find name obj when set global, in line {}", line);
                    };

                    let Ok(_) = self.memory.set_global(&name_str, new_val) else {
                        bail!("undefined variable {}, in line {}", name_str, line);
                    };
                }
                OpCode::GetLocal => {
                    let slot = self.read_byte() as usize;
                    self.advance_ip();
                    let stack_base = self.current_frame().stack_base;
                    let Some(v) = self.memory.stack_get(stack_base + slot).cloned() else {
                        bail!("not find local in {}, in line {}", slot, line);
                    };

                    self.memory.stack_push(v);
                }
                OpCode::SetLocal => {
                    let slot = self.read_byte() as usize;
                    self.advance_ip();
                    let stack_base = self.current_frame().stack_base;
                    let value = self.memory.stack_peek().clone();
                    self.memory.stack_set(stack_base + slot, value);
                }
                OpCode::JumpIfFalse => {
                    let offset = self.get_offset();
                    if !self.memory.stack_peek().to_bool(&self.memory) {
                        self.frames.last_mut().unwrap().ip += offset;
                    }
                }
                OpCode::Jump => {
                    let offset = self.get_offset();
                    self.frames.last_mut().unwrap().ip += offset;
                }
                OpCode::RevJump => {
                    let offset = self.get_offset();
                    self.frames.last_mut().unwrap().ip -= offset;
                }
            }
        }
    }
}

impl<W: Write> std::fmt::Display for VM<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = Builder::new();
        if let Some(frame) = self.frames.last() {
            builder.push_column([
                "chunk".to_string(),
                format!("{}", frame.chunk.with_ip(frame.ip as i32)),
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
                Some(o) => {
                    let s = match o.kind.to_string() {
                        Ok(m) => m,
                        Err(_) => "<err>".to_string(),
                    };
                    format!("[{addr}]: {}", s)
                }
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
