use std::{collections::HashMap, sync::LazyLock};

use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};

use crate::error::LoxError;

#[derive(Display, Clone, PartialEq, Debug, Eq, Hash, Copy, EnumCount, EnumIter)]
#[repr(usize)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Function,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

impl Into<usize> for TokenType {
    fn into(self) -> usize {
        return self as usize;
    }
}

impl TryFrom<usize> for TokenType {
    type Error = LoxError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        for typ in TokenType::iter() {
            if value == typ.clone().into() {
                return Ok(typ);
            }
        }

        return Err(LoxError::ChunkError(
            "Fail to convert usize to TokenType".into(),
        ));
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Token {
    pub(crate) typ: TokenType,
    pub(crate) lexeme: String,
    pub(crate) line: usize,
}

pub static KEYWORDS: LazyLock<HashMap<String, TokenType>> = LazyLock::new(|| {
    let mut m = HashMap::with_capacity(16);
    m.insert("and".into(), TokenType::And);
    m.insert("class".into(), TokenType::Class);
    m.insert("else".into(), TokenType::Else);
    m.insert("false".into(), TokenType::False);
    m.insert("for".into(), TokenType::For);
    m.insert("fun".into(), TokenType::Function);
    m.insert("if".into(), TokenType::If);
    m.insert("nil".into(), TokenType::Nil);
    m.insert("or".into(), TokenType::Or);
    m.insert("print".into(), TokenType::Print);
    m.insert("return".into(), TokenType::Return);
    m.insert("super".into(), TokenType::Super);
    m.insert("this".into(), TokenType::This);
    m.insert("true".into(), TokenType::True);
    m.insert("var".into(), TokenType::Var);
    m.insert("while".into(), TokenType::While);
    m
});
