use core::fmt;
use std::cell::RefCell;

use crate::model::{opcode::OpCode, value::Constant};

#[derive(Clone)]
struct LineStart {
    offset: usize,
    line: usize,
}

#[derive(Clone)]
pub struct Chunk {
    pub(crate) code: RefCell<Vec<u8>>,
    pub(crate) constants: RefCell<Vec<Constant>>,
    lines: RefCell<Vec<LineStart>>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: RefCell::new(Vec::new()),
            constants: RefCell::new(Vec::new()),
            lines: RefCell::new(Vec::new()),
        }
    }

    pub fn write<T: Into<u8>>(&self, t: T, line: usize) {
        let val = t.into();
        self.code.borrow_mut().push(val);

        let is_new_line =
            self.lines.borrow().is_empty() || self.lines.borrow().last().unwrap().line != line;
        if is_new_line {
            self.lines.borrow_mut().push(LineStart {
                offset: self.code.borrow().len() - 1,
                line,
            });
        }
    }

    pub fn get_line(&self, instruction_offset: usize) -> usize {
        // 找到第一个 offset 大于 instruction_offset 的位置
        let index = self
            .lines
            .borrow()
            .partition_point(|x| x.offset <= instruction_offset);

        // 该位置的前一个元素就是目标行号
        if index > 0 {
            self.lines.borrow()[index - 1].line
        } else {
            self.lines.borrow()[0].line
        }
    }

    pub fn add_constant(&self, v: Constant) -> u8 {
        let index = self.constants.borrow().len();
        self.constants.borrow_mut().push(v);
        if index < 256 {
            return index as u8;
        }
        panic!("Too many constants in one chunk!");
    }

    pub fn with_ip(&self, ip: i32) -> ChunkWithIp<'_> {
        ChunkWithIp { ip, chunk: &self }
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.with_ip(-1))
    }
}

pub struct ChunkWithIp<'a> {
    pub ip: i32,
    pub chunk: &'a Chunk,
}

impl<'a> fmt::Display for ChunkWithIp<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut offset: usize = 0;

        while offset < self.chunk.code.borrow().len() {
            let code = self.chunk.code.borrow()[offset];
            let is_highlight = offset as i32 == self.ip;
            if is_highlight {
                write!(f, " -> ")?;
            } else {
                write!(f, "    ")?;
            }
            match OpCode::try_from(code) {
                Ok(code) => match code {
                    OpCode::Constant
                    | OpCode::DefineGlobal
                    | OpCode::GetGlobal
                    | OpCode::SetGlobal
                    | OpCode::GetLocal
                    | OpCode::SetLocal => {
                        write!(f, "{:04} ", offset)?;

                        let value_index = self.chunk.code.borrow()[offset + 1] as usize;
                        let constants = self.chunk.constants.borrow();
                        let value = constants.get(value_index);
                        if let Some(v) = value {
                            write!(f, "{}({})", code, v)?;
                        } else {
                            panic!("invalid constant index");
                        }
                        offset += 2;
                    }
                    _ => {
                        write!(f, "{:04} ", offset)?;
                        write!(f, "{}", code)?;

                        offset += 1;
                    }
                },
                Err(_) => {
                    panic!("invalid op code: {}", code)
                }
            };

            if offset < self.chunk.code.borrow().len() {
                writeln!(f, "")?;
            }
        }

        Ok(())
    }
}
