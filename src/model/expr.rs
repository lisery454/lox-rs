use std::fmt::Display;

use colored::Colorize;

use crate::model::{literal::LiteralValue, token::Token};

#[derive(Debug, Clone)]
pub struct AssignExprData {
    pub(crate) name: Token,
    pub(crate) value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct BinaryExprData {
    pub(crate) operator: Token,
    pub(crate) left: Box<Expr>,
    pub(crate) right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct CallExprData {
    pub(crate) operator: Token,
    pub(crate) callee: Box<Expr>,
    pub(crate) arguments: Vec<Box<Expr>>,
}

#[derive(Debug, Clone)]
pub struct GetExprData {
    pub(crate) name: Token,
    pub(crate) object: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct GroupingExprData {
    pub(crate) expression: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct LiteralExprData {
    pub(crate) value: LiteralValue,
}

#[derive(Debug, Clone)]
pub struct LogicalExprData {
    pub(crate) left: Box<Expr>,
    pub(crate) right: Box<Expr>,
    pub(crate) operator: Token,
}

#[derive(Debug, Clone)]
pub struct SetExprData {
    pub(crate) name: Token,
    pub(crate) object: Box<Expr>,
    pub(crate) value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct SuperExprData {
    pub(crate) keyword: Token,
    pub(crate) method: Token,
}

#[derive(Debug, Clone)]
pub struct ThisExprData {
    pub(crate) keyword: Token,
}

#[derive(Debug, Clone)]
pub struct UnaryExprData {
    pub(crate) operator: Token,
    pub(crate) right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct VariableExprData {
    pub(crate) name: Token,
}

#[derive(Debug, Clone)]
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
    pub fn assign(name: Token, value: Expr) -> Expr {
        Expr::Assign(AssignExprData {
            name,
            value: Box::new(value),
        })
    }

    pub fn binary(op: Token, left: Expr, right: Expr) -> Expr {
        Expr::Binary(BinaryExprData {
            operator: op,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    pub fn call(op: Token, callee: Expr, arguments: Vec<Expr>) -> Expr {
        Expr::Call(CallExprData {
            operator: op,
            callee: Box::new(callee),
            arguments: arguments.into_iter().map(|e| Box::new(e)).collect(),
        })
    }

    pub fn get(name: Token, object: Expr) -> Expr {
        Expr::Get(GetExprData {
            name,
            object: Box::new(object),
        })
    }

    pub fn grouping(expr: Expr) -> Expr {
        Expr::Grouping(GroupingExprData {
            expression: Box::new(expr),
        })
    }

    pub fn literal<T>(v: T) -> Expr
    where
        T: Into<LiteralValue>,
    {
        Expr::Literal(LiteralExprData { value: v.into() })
    }

    pub fn logical(op: Token, left: Expr, right: Expr) -> Expr {
        Expr::Logical(LogicalExprData {
            left: Box::new(left),
            right: Box::new(right),
            operator: op,
        })
    }

    pub fn set(name: Token, object: Expr, value: Expr) -> Expr {
        Expr::Set(SetExprData {
            name,
            object: Box::new(object),
            value: Box::new(value),
        })
    }

    pub fn super_(keyword: Token, method: Token) -> Expr {
        Expr::Super(SuperExprData { keyword, method })
    }

    pub fn this(keyword: Token) -> Expr {
        Expr::This(ThisExprData { keyword })
    }

    pub fn unary(op: Token, right: Expr) -> Expr {
        Expr::Unary(UnaryExprData {
            right: Box::new(right),
            operator: op,
        })
    }

    pub fn variable(name: Token) -> Expr {
        Expr::Variable(VariableExprData { name })
    }
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
