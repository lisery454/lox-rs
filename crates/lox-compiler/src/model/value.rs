use std::fmt::{self};

pub enum Obj {
    String(String),
}

impl fmt::Display for Obj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Obj::String(s) => writeln!(f, "{s}"),
        }
    }
}

/// use in VM, is dynamic, need memory management
#[derive(Clone)]
#[repr(C)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    Nil,
    Obj(*mut Obj),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::Nil => write!(f, "<nil>"),
            Value::Obj(obj) => unsafe {
                let o = &**obj;
                match o {
                    Obj::String(s) => write!(f, "{}", s),
                }
            },
        }
    }
}

/// use in Chunk, is static
#[derive(Clone)]
#[repr(C)]
pub enum Constant {
    Number(f64),
    String(String),
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Constant::Number(n) => write!(f, "{n}"),
            Constant::String(s) => write!(f, "{}", s),
        }
    }
}
