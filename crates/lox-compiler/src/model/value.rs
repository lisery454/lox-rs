use std::{
    fmt::{self, write},
    rc::Rc,
};

#[repr(C)]
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
            Value::Obj(obj) => write!(f, "<obj {:?}>", obj),
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
