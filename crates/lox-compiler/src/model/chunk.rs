use core::fmt;
use std::cell::RefCell;

use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

use crate::{
    error::LoxError,
    model::value::{Value, ValueArray},
};

#[derive(Display, EnumIter, PartialEq, Clone)]
#[repr(u8)]
pub enum OpCode {
    Constant,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Return,
}

impl TryFrom<u8> for OpCode {
    type Error = LoxError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        for code in OpCode::iter() {
            if value == code.clone().into() {
                return Ok(code);
            }
        }

        return Err(LoxError::ChunkError("Fail to convert u8 to OpCode".into()));
    }
}

impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        return self as u8;
    }
}

struct LineStart {
    offset: usize,
    line: u32,
}

pub struct Chunk {
    pub(crate) code: RefCell<Vec<u8>>,
    pub(crate) constants: RefCell<ValueArray>,
    lines: RefCell<Vec<LineStart>>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: RefCell::new(Vec::new()),
            constants: RefCell::new(ValueArray::new()),
            lines: RefCell::new(Vec::new()),
        }
    }

    pub fn write<T: Into<u8>>(&self, t: T, line: u32) {
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

    pub fn get_line(&self, instruction_offset: usize) -> u32 {
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

    pub fn add_constant(&self, v: Value) -> u8 {
        let index = self.constants.borrow_mut().write(v);
        if index < 256 {
            return index as u8;
        }
        panic!("Too many constants in one chunk!");
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut offset: usize = 0;
        while offset < self.code.borrow().len() {
            let code = self.code.borrow()[offset];

            match OpCode::try_from(code) {
                Ok(code) => match code {
                    OpCode::Constant => {
                        write!(f, "{:04} ", offset)?;
                        let value_index = self.code.borrow()[offset + 1] as usize;
                        let constants = self.constants.borrow();
                        let value = constants.read(value_index);
                        if let Some(v) = value {
                            writeln!(f, "{}({})", code, v)?;
                        } else {
                            panic!("invalid constant index");
                        }
                        offset += 2;
                    }
                    _ => {
                        write!(f, "{:04} ", offset)?;
                        writeln!(f, "{}", code)?;
                        offset += 1;
                    }
                },
                Err(_) => {
                    panic!("invalid op code: {}", code)
                }
            };
        }

        Ok(())
    }
}
