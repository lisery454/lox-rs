use std::num::ParseFloatError;

use thiserror::Error;

pub type LoxResult<T> = std::result::Result<T, LoxError>;

#[derive(Error, Debug)]
pub enum LoxError {
    #[error("chunk error: {0}")]
    ChunkError(String),

    #[error("runtime error: [line {line}] Error, {message}")]
    RuntimeError{
        line: usize,
        message: String,
    },

    #[error("compile error: {0}")]
    CompileError(String),

    #[error("scan error: [line {line}] Error at '{lexeme}', {message}")]
    ScanError {
        lexeme: String,
        line: usize,
        message: String,
    },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Format error: {0}")]
    FmtError(#[from] std::fmt::Error),

    #[error("Parse error: {0}")]
    ParseError(#[from] ParseFloatError),

    #[error("merge error: {}", errors.get(0).unwrap())]
    MergeError { errors: Vec<LoxError> },
}
