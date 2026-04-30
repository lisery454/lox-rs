use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

impl LiteralValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            LiteralValue::Number(n) => {
                if n.abs() > f64::EPSILON {
                    true
                } else {
                    true
                }
            }
            LiteralValue::String(s) => {
                if s.len() > 0 {
                    true
                } else {
                    false
                }
            }
            LiteralValue::Bool(b) => {
                if *b {
                    true
                } else {
                    false
                }
            }
            LiteralValue::Nil => false,
        }
    }
}

impl Display for LiteralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteralValue::Number(n) => write!(f, "{}", n)?,
            LiteralValue::String(s) => write!(f, "{}", s)?,
            LiteralValue::Bool(b) => write!(f, "{}", b)?,
            LiteralValue::Nil => write!(f, "nil",)?,
        }
        Ok(())
    }
}

impl From<String> for LiteralValue {
    fn from(s: String) -> Self {
        LiteralValue::String(s)
    }
}

impl From<f64> for LiteralValue {
    fn from(n: f64) -> Self {
        LiteralValue::Number(n)
    }
}

impl From<bool> for LiteralValue {
    fn from(b: bool) -> Self {
        LiteralValue::Bool(b)
    }
}

impl From<()> for LiteralValue {
    fn from(_: ()) -> Self {
        LiteralValue::Nil
    }
}
