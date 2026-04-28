use std::fmt::Display;

use anyhow::Result;
use colored::Colorize;

use crate::{error::LoxError, model::token::TokenType};

use super::token::Token;

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
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

#[derive(Debug)]
pub struct AssignExprData {
    pub(crate) name: Token,
    pub(crate) value: Box<Expr>,
}

#[derive(Debug)]
pub struct BinaryExprData {
    pub(crate) operator: Token,
    pub(crate) left: Box<Expr>,
    pub(crate) right: Box<Expr>,
}

#[derive(Debug)]
pub struct CallExprData {
    pub(crate) operator: Token,
    pub(crate) callee: Box<Expr>,
    pub(crate) arguments: Vec<Box<Expr>>,
}

#[derive(Debug)]
pub struct GetExprData {
    pub(crate) name: Token,
    pub(crate) object: Box<Expr>,
}

#[derive(Debug)]
pub struct GroupingExprData {
    pub(crate) expression: Box<Expr>,
}

#[derive(Debug)]
pub struct LiteralExprData {
    pub(crate) value: LiteralValue,
}

#[derive(Debug)]
pub struct LogicalExprData {
    pub(crate) left: Box<Expr>,
    pub(crate) right: Box<Expr>,
    pub(crate) operator: Token,
}

#[derive(Debug)]
pub struct SetExprData {
    pub(crate) name: Token,
    pub(crate) object: Box<Expr>,
    pub(crate) value: Box<Expr>,
}

#[derive(Debug)]
pub struct SuperExprData {
    pub(crate) keyword: Token,
    pub(crate) method: Token,
}

#[derive(Debug)]
pub struct ThisExprData {
    pub(crate) keyword: Token,
}

#[derive(Debug)]
pub struct UnaryExprData {
    pub(crate) operator: Token,
    pub(crate) right: Box<Expr>,
}

#[derive(Debug)]
pub struct VariableExprData {
    pub(crate) name: Token,
}

#[derive(Debug)]
pub enum Expr {
    Assign(AssignExprData),
    Binary(BinaryExprData),
    Call(CallExprData),
    Get(GetExprData),
    Grouping(GroupingExprData),
    Literal(LiteralExprData),
    Logical(LogicalExprData),
    Set(SetExprData),
    Super(SuperExprData),
    This(ThisExprData),
    Unary(UnaryExprData),
    Variable(VariableExprData),
}

impl Expr {
    fn render(&self, f: &mut std::fmt::Formatter<'_>, depth: usize) -> std::fmt::Result {
        let indent = "    ".repeat(depth + 1);
        let title = |s: &str| return s.bold().color("green");
        match self {
            Expr::Assign(assign_expr_data) => {
                writeln!(f, "[{}] name: {}", title("assign"), assign_expr_data.name)?;
                write!(f, "{}value: ", indent)?;
                assign_expr_data.value.render(f, depth + 1)?;
                Ok(())
            }
            Expr::Binary(binary_expr_data) => {
                writeln!(f, "[{}] op: {}", title("binary"), binary_expr_data.operator)?;
                write!(f, "{}left: ", indent)?;
                binary_expr_data.left.render(f, depth + 1)?;
                write!(f, "{}right: ", indent)?;
                binary_expr_data.right.render(f, depth + 1)?;
                Ok(())
            }
            Expr::Call(call_expr_data) => {
                writeln!(f, "[{}] op: {}", title("call"), call_expr_data.operator)?;
                write!(f, "{}callee: ", indent)?;
                call_expr_data.callee.render(f, depth + 1)?;
                write!(f, "{}args: ", indent)?;
                for arg in &call_expr_data.arguments {
                    arg.render(f, depth + 1)?;
                }
                Ok(())
            }
            Expr::Get(get_expr_data) => {
                writeln!(f, "[{}] name: {}", title("get"), get_expr_data.name)?;
                write!(f, "{}object: ", indent)?;
                get_expr_data.object.render(f, depth + 1)?;
                Ok(())
            }
            Expr::Grouping(grouping_expr_data) => {
                writeln!(f, "[{}]", title("grouping"))?;
                write!(f, "{}exp: ", indent)?;
                grouping_expr_data.expression.render(f, depth + 1)?;
                Ok(())
            }
            Expr::Literal(literal_expr_data) => {
                writeln!(f, "[{}]", title("literal"))?;
                write!(f, "{}value: ", indent)?;
                writeln!(f, "{}", literal_expr_data.value)?;
                Ok(())
            }
            Expr::Logical(logical_expr_data) => {
                writeln!(
                    f,
                    "[{}] op: {}",
                    title("logical"),
                    logical_expr_data.operator
                )?;
                write!(f, "{}left: ", indent)?;
                logical_expr_data.left.render(f, depth + 1)?;
                write!(f, "{}right: ", indent)?;
                logical_expr_data.right.render(f, depth + 1)?;
                Ok(())
            }
            Expr::Set(set_expr_data) => {
                writeln!(f, "[{}] name: {}", title("set"), set_expr_data.name)?;
                write!(f, "{}object: ", indent)?;
                set_expr_data.object.render(f, depth + 1)?;
                write!(f, "{}value: ", indent)?;
                set_expr_data.value.render(f, depth + 1)?;
                Ok(())
            }
            Expr::Super(super_expr_data) => {
                writeln!(
                    f,
                    "[{}] keyword: {}, methods: {}",
                    title("super"),
                    super_expr_data.keyword,
                    super_expr_data.method
                )?;
                Ok(())
            }
            Expr::This(this_expr_data) => {
                writeln!(f, "[{}] keyword: {}", title("this"), this_expr_data.keyword)?;
                Ok(())
            }
            Expr::Unary(unary_expr_data) => {
                writeln!(f, "[{}] op: {}", title("unary"), unary_expr_data.operator)?;
                write!(f, "{}right: ", indent)?;
                unary_expr_data.right.render(f, depth + 1)?;
                Ok(())
            }
            Expr::Variable(variable_expr_data) => {
                writeln!(
                    f,
                    "[{}] name: {}",
                    title("variable"),
                    variable_expr_data.name
                )?;
                Ok(())
            }
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.render(f, 0)
    }
}

impl Expr {
    pub fn interpret(&self) -> Result<LiteralValue> {
        match self {
            Expr::Assign(_assign_expr_data) => Ok(LiteralValue::Nil),
            Expr::Binary(binary_expr_data) => {
                let left_value = binary_expr_data.left.interpret()?;
                let right_value = binary_expr_data.right.interpret()?;
                match binary_expr_data.operator.typ {
                    TokenType::Minus => {
                        if let LiteralValue::Number(l) = &left_value
                            && let LiteralValue::Number(r) = &right_value
                        {
                            return Ok(LiteralValue::Number(l - r));
                        }
                    }
                    TokenType::Plus => {
                        if let LiteralValue::Number(l) = &left_value
                            && let LiteralValue::Number(r) = &right_value
                        {
                            return Ok(LiteralValue::Number(l + r));
                        } else if let LiteralValue::String(l) = left_value
                            && let LiteralValue::String(r) = right_value
                        {
                            return Ok(LiteralValue::String(format!("{}{}", l, r)));
                        }
                    }
                    TokenType::Slash => {
                        if let LiteralValue::Number(l) = &left_value
                            && let LiteralValue::Number(r) = &right_value
                        {
                            return Ok(LiteralValue::Number(l / r));
                        }
                    }
                    TokenType::Star => {
                        if let LiteralValue::Number(l) = &left_value
                            && let LiteralValue::Number(r) = &right_value
                        {
                            return Ok(LiteralValue::Number(l * r));
                        }
                    }
                    TokenType::Greater => {
                        if let LiteralValue::Number(l) = &left_value
                            && let LiteralValue::Number(r) = &right_value
                        {
                            return Ok(LiteralValue::Bool(l > r));
                        }
                    }
                    TokenType::GreaterEqual => {
                        if let LiteralValue::Number(l) = &left_value
                            && let LiteralValue::Number(r) = &right_value
                        {
                            return Ok(LiteralValue::Bool(l >= r));
                        }
                    }
                    TokenType::Less => {
                        if let LiteralValue::Number(l) = &left_value
                            && let LiteralValue::Number(r) = &right_value
                        {
                            return Ok(LiteralValue::Bool(l < r));
                        }
                    }
                    TokenType::LessEqual => {
                        if let LiteralValue::Number(l) = &left_value
                            && let LiteralValue::Number(r) = &right_value
                        {
                            return Ok(LiteralValue::Bool(l <= r));
                        }
                    }
                    TokenType::EqualEqual => {
                        if let LiteralValue::Number(l) = &left_value
                            && let LiteralValue::Number(r) = &right_value
                        {
                            return Ok(LiteralValue::Bool(l == r));
                        } else if let LiteralValue::String(l) = &left_value
                            && let LiteralValue::String(r) = &right_value
                        {
                            return Ok(LiteralValue::Bool(l == r));
                        } else if let LiteralValue::Bool(l) = &left_value
                            && let LiteralValue::Bool(r) = &right_value
                        {
                            return Ok(LiteralValue::Bool(l == r));
                        } else if let LiteralValue::Nil = &left_value
                            && let LiteralValue::Nil = &right_value
                        {
                            return Ok(LiteralValue::Bool(true));
                        }
                    }
                    TokenType::BangEqual => {
                        if let LiteralValue::Number(l) = &left_value
                            && let LiteralValue::Number(r) = &right_value
                        {
                            return Ok(LiteralValue::Bool(l != r));
                        } else if let LiteralValue::String(l) = &left_value
                            && let LiteralValue::String(r) = &right_value
                        {
                            return Ok(LiteralValue::Bool(l != r));
                        } else if let LiteralValue::Bool(l) = &left_value
                            && let LiteralValue::Bool(r) = &right_value
                        {
                            return Ok(LiteralValue::Bool(l != r));
                        } else if let LiteralValue::Nil = &left_value
                            && let LiteralValue::Nil = &right_value
                        {
                            return Ok(LiteralValue::Bool(false));
                        }
                    }
                    _ => {}
                }
                Err(LoxError::InterpretError {
                    message: format!(
                        "calc binary expr fail. line: {}. lexeme: {}",
                        binary_expr_data.operator.line, binary_expr_data.operator.lexeme
                    ),
                }
                .into())
            }
            Expr::Call(_call_expr_data) => Ok(LiteralValue::Nil),
            Expr::Get(_get_expr_data) => Ok(LiteralValue::Nil),
            Expr::Grouping(grouping_expr_data) => grouping_expr_data.expression.interpret(),
            Expr::Literal(literal_expr_data) => Ok(literal_expr_data.value.clone()),
            Expr::Logical(_logical_expr_data) => Ok(LiteralValue::Nil),
            Expr::Set(_set_expr_data) => Ok(LiteralValue::Nil),
            Expr::Super(_super_expr_data) => Ok(LiteralValue::Nil),
            Expr::This(_this_expr_data) => Ok(LiteralValue::Nil),
            Expr::Unary(unary_expr_data) => {
                let right_value = unary_expr_data.right.interpret()?;
                match unary_expr_data.operator.typ {
                    TokenType::Minus => {
                        if let LiteralValue::Number(n) = &right_value {
                            return Ok(LiteralValue::Number(-n));
                        }
                    }
                    TokenType::Bang => {
                        if let LiteralValue::Bool(n) = &right_value {
                            return Ok(LiteralValue::Bool(!n));
                        }
                    }
                    _ => {}
                }
                Err(LoxError::InterpretError {
                    message: format!(
                        "calc unary expr fail. line: {}. lexeme: {}",
                        unary_expr_data.operator.line, unary_expr_data.operator.lexeme
                    ),
                }
                .into())
            }
            Expr::Variable(_variable_expr_data) => Ok(LiteralValue::Nil),
        }
    }
}
