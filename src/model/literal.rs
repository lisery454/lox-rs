use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::environment::Environment;

type NativeFunction = fn(Rc<RefCell<Environment>>, Vec<LiteralValue>) -> LiteralValue;

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Number(f64),
    String(String),
    Bool(bool),
    Callable {
        function: NativeFunction,
        arg_size: usize,
    },
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
            LiteralValue::Callable {
                arg_size: _arg_size,
                function: _function,
            } => true,
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
            LiteralValue::Callable {
                arg_size: _arg_size,
                function: _function,
            } => write!(f, "callable",)?,
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
