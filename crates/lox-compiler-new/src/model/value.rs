use crate::model::{Memory, ObjAddr, ObjectKind};

/// use in VM, is dynamic, need memory management
#[derive(Clone, Copy)]
#[repr(C)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    Object(ObjAddr),
    Nil,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::Object(addr) => write!(f, "<addr:{addr}>"),
            Value::Nil => write!(f, "<nil>"),
        }
    }
}

impl Value {
    pub fn print(&self, memory: &Memory) {
        match self {
            Value::Boolean(b) => println!("{}", b),
            Value::Number(n) => println!("{}", n),
            Value::Nil => println!("<nil>"),
            Value::Object(addr) => match memory.get_obj(*addr) {
                Some(kind) => match kind {
                    ObjectKind::String(s) => println!("{}", s),
                },
                None => {
                    println!("<nil>")
                }
            },
        }
    }

    pub fn to_bool(&self, memory: &Memory) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Number(n) => {
                if *n == 0.0 {
                    false
                } else {
                    true
                }
            }
            Value::Nil => false,
            Value::Object(addr) => match memory.get_obj(*addr) {
                Some(_) => true,
                None => false,
            },
        }
    }
}
