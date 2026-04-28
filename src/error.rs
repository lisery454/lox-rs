use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoxError {
    // #[error("[line {}] Error {}: {}", line, loc, message)]
    // LineError {
    //     line: u32,
    //     loc: String,
    //     message: String,
    // },
    #[error("not found char")]
    ReadCharNotFound,
    #[error("unexpected character, line: {line}")]
    UnexpectedChar { line: u32 },
    #[error("unterminated string, line: {line}")]
    UnterminatedString { line: u32 },
    #[error("invalid number format, line: {line}")]
    InvalidNumberFormat { line: u32 },

    #[error("parse error: {message}")]
    ParseError { message: String },

    #[error("interpret error: {message}")]
    InterpretError { message: String },
}
