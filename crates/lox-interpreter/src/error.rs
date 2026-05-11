use thiserror::Error;

use crate::model::literal::LiteralValue;

pub type LoxResult<T> = std::result::Result<T, LoxError>;

#[derive(Error, Debug)]
pub enum LoxError {
    #[error("scan error: {message}")]
    ScanError { message: String },

    #[error("parse error: {message}")]
    ParseError { message: String },

    #[error("interpret error: {message}")]
    InterpretError { message: String },

    #[error("return error")]
    ReturnError(LiteralValue),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Format error: {0}")]
    FmtError(#[from] std::fmt::Error),
}
