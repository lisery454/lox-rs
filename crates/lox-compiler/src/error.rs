use thiserror::Error;

pub type LoxResult<T> = std::result::Result<T, LoxError>;

#[derive(Error, Debug)]
pub enum LoxError {
    #[error("chunk error: {0}")]
    ChunkError(String),

    #[error("runtime error: {0}")]
    RuntimeError(String),

    #[error("compile error: {0}")]
    CompileError(String),

    #[error("scan error: {0}")]
    ScanError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Format error: {0}")]
    FmtError(#[from] std::fmt::Error),
}
