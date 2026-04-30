use std::{cell::RefCell, rc::Rc};

use crate::{
    environment::Environment,
    error::LoxError,
    model::{expr::Expr, literal::LiteralValue, stmt::Stmt, token::TokenType},
};

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn interpret(&mut self, stmt: &Stmt) -> anyhow::Result<()> {
        match stmt {
            Stmt::Block(block_stmt_data) => {
                let prev_env = self.environment.clone();
                let new_env = Rc::new(RefCell::new(Environment::new_with_parent(&prev_env)));
                {
                    self.environment = new_env;
                    for stmt in &block_stmt_data.statements {
                        self.interpret(&stmt)?;
                    }
                    self.environment = prev_env;
                }
                Ok(())
            }
            Stmt::Class(_class_stmt_data) => Ok(()),
            Stmt::Expression(expression_stmt_data) => {
                self.interpret_expr(&expression_stmt_data.expression)?;
                Ok(())
            }
            Stmt::Function(_function_stmt_data) => Ok(()),
            Stmt::If(if_stmt_data) => {
                let condition = self.interpret_expr(&if_stmt_data.condition)?;
                if condition.is_truthy() {
                    if let Some(then_branch) = &if_stmt_data.then_branch {
                        self.interpret(then_branch)?;
                    }
                } else {
                    if let Some(else_branch) = &if_stmt_data.else_branch {
                        self.interpret(else_branch)?;
                    }
                }
                Ok(())
            }
            Stmt::Print(print_stmt_data) => {
                let v = self.interpret_expr(&print_stmt_data.expression)?;
                println!("{}", v);
                Ok(())
            }
            Stmt::Return(_return_stmt_data) => Ok(()),
            Stmt::Variable(variable_stmt_data) => {
                let value = if let Some(initializer) = &variable_stmt_data.initializer {
                    self.interpret_expr(initializer)?
                } else {
                    LiteralValue::Nil
                };
                self.environment
                    .borrow_mut()
                    .define(&variable_stmt_data.name.lexeme, value);

                Ok(())
            }
            Stmt::While(_while_stmt_data) => Ok(()),
        }
    }

    pub fn interpret_expr(&mut self, expr: &Expr) -> anyhow::Result<LiteralValue> {
        match expr {
            Expr::Assign(data) => {
                let value = self.interpret_expr(&data.value)?;
                let success = self
                    .environment
                    .borrow_mut()
                    .assign(&data.name.lexeme, value);
                if success {
                    Ok(LiteralValue::Nil)
                } else {
                    Err(LoxError::InterpretError {
                        message: format!(
                            "can't assign to undefined variable. line: {}. lexeme: {}",
                            data.name.line, data.name.lexeme
                        ),
                    }
                    .into())
                }
            }
            Expr::Binary(data) => {
                let left_value = self.interpret_expr(&data.left)?;
                let right_value = self.interpret_expr(&data.right)?;
                match data.operator.typ {
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
                        } else if let LiteralValue::String(l) = &left_value
                            && let LiteralValue::String(r) = &right_value
                        {
                            return Ok(LiteralValue::String(format!("{}{}", l, r)));
                        } else if let LiteralValue::String(l) = &left_value
                            && let LiteralValue::Number(r) = &right_value
                        {
                            return Ok(LiteralValue::String(format!("{}{}", l, r)));
                        } else if let LiteralValue::Number(l) = &left_value
                            && let LiteralValue::String(r) = &right_value
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
                        return Ok(LiteralValue::Bool(false));
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
                        return Ok(LiteralValue::Bool(true));
                    }
                    _ => {}
                }
                Err(LoxError::InterpretError {
                    message: format!(
                        "calc binary expr fail. line: {}. lexeme: {}",
                        data.operator.line, data.operator.lexeme
                    ),
                }
                .into())
            }
            Expr::Call(_data) => Ok(LiteralValue::Nil),
            Expr::Get(_data) => Ok(LiteralValue::Nil),
            Expr::Grouping(data) => self.interpret_expr(&data.expression),
            Expr::Literal(data) => Ok(data.value.clone()),
            Expr::Logical(_data) => Ok(LiteralValue::Nil),
            Expr::Set(_data) => Ok(LiteralValue::Nil),
            Expr::Super(_data) => Ok(LiteralValue::Nil),
            Expr::This(_data) => Ok(LiteralValue::Nil),
            Expr::Unary(data) => {
                let right_value = self.interpret_expr(&data.right)?;
                match data.operator.typ {
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
                        data.operator.line, data.operator.lexeme
                    ),
                }
                .into())
            }
            Expr::Variable(data) => {
                let value = self.environment.borrow().get(&data.name.lexeme);
                Ok(value)
            }
        }
    }
}
