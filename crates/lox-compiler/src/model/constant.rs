use crate::model::Function;

/// use in Chunk, is static
#[derive(Clone)]
#[repr(C)]
pub enum Constant {
    Number(f64),
    String(String),
    Function(Function),
}

impl std::fmt::Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant::Number(n) => write!(f, "num<{}>", n),
            Constant::String(s) => write!(f, "str<{}>", s),
            Constant::Function(func) => write!(f, "fn<{}>", func.name),
        }
    }
}
