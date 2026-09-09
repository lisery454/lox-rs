/// use in VM, is dynamic, need memory management
#[derive(Clone)]
#[repr(C)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    Nil,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::Nil => write!(f, "<nil>"),
        }
    }
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Boolean(b) => !*b,
            Value::Number(n) => {
                if *n == 0.0 {
                    true
                } else {
                    false
                }
            }
            Value::Nil => true,
        }
    }
}
