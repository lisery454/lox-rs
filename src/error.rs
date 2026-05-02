use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoxError {
    #[error("scan error: {message}")]
    ScanError { message: String },

    #[error("parse error: {message}")]
    ParseError { message: String },

    #[error("interpret error: {message}")]
    InterpretError { message: String },
}
