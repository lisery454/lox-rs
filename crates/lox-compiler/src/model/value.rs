use std::fmt;

#[derive(Clone, Copy)]
#[repr(C)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    Nil,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::Nil => write!(f, "<nil>"),
        }
    }
}

#[derive(Clone)]
pub struct ValueArray {
    pub(crate) values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn read(&self, i: usize) -> Option<&Value> {
        self.values.get(i)
    }

    pub fn write(&mut self, t: Value) -> usize {
        self.values.push(t);
        self.values.len() - 1
    }
}
