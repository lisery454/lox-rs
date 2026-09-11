use num_enum::TryFromPrimitive;
use strum::{Display, EnumIter};

#[derive(Display, EnumIter, PartialEq, Clone, TryFromPrimitive)]
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
    RevJump,
    Jump,
    Call,
    Return,
}

impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        return self as u8;
    }
}
