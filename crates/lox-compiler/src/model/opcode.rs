use strum::{Display, EnumIter, IntoEnumIterator};

use crate::error::LoxError;

#[derive(Display, EnumIter, PartialEq, Clone)]
#[repr(u8)]
pub enum OpCode {
    Constant,
    Nil,
    True,
    False,
    Pop,
    GetGlobal,
    SetGlobal,
    GetLocal,
    SetLocal,
    DefineGlobal,
    Equal,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,
    Print,
    JumpIfFalse,
    Jump,
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