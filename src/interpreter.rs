use std::{cell::RefCell, rc::Rc};

use chrono::{DateTime, Local};

use crate::{
    environment::Environment,
    error::LoxError,
    model::{expr::Expr, literal::LiteralValue, stmt::Stmt, token::TokenType},
};

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
    global_env: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let environment = Rc::new(RefCell::new(Environment::new()));
        let result = Self {
            environment: environment.clone(),
            global_env: environment.clone(),
        };

        result.global_env.borrow_mut().define(
            &"clock".to_string(),
            LiteralValue::Callable {
                function: |_env, _args| {
                    let now: DateTime<Local> = Local::now();
                    LiteralValue::String(now.format("%Y-%m-%d %H:%M:%S").to_string())
                },
                arg_size: 0,
            },
        );

        result.global_env.borrow_mut().define(
            &"writeln".to_string(),
            LiteralValue::Callable {
                function: |_env, args| {
                    println!("{}", args[0]);
                    LiteralValue::Nil
                },
                arg_size: 1,
            },
        );

        result.global_env.borrow_mut().define(
            &"write".to_string(),
            LiteralValue::Callable {
                function: |_env, args| {
                    print!("{}", args[0]);
                    LiteralValue::Nil
                },
                arg_size: 1,
            },
        );

        result
    }

    pub fn interpret(&mut self, stmt: &Stmt) -> anyhow::Result<()> {
        interpret(self.environment.clone(), stmt)
    }
}

fn interpret_expr(
    environment: Rc<RefCell<Environment>>,
    expr: &Expr,
) -> anyhow::Result<LiteralValue> {
    match expr {
        Expr::Assign(data) => {
            let value = interpret_expr(environment.clone(), &data.value)?;
            let success = environment.borrow_mut().assign(&data.name.lexeme, value);
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
            let left_value = interpret_expr(environment.clone(), &data.left)?;
            let right_value = interpret_expr(environment.clone(), &data.right)?;
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
        Expr::Call(data) => {
            let callable = interpret_expr(environment.clone(), &data.callee)?;
            let mut arguments = Vec::new();
            for argument in &data.arguments {
                let arg = interpret_expr(environment.clone(), &argument)?;
                arguments.push(arg);
            }
            if let LiteralValue::Callable { function, arg_size } = callable {
                if arguments.len() != arg_size {
                    return Err(LoxError::InterpretError {
                        message: format!(
                            "Expected {} arguments but got {}. line: {}. lexeme: {}",
                            arg_size,
                            arguments.len(),
                            data.operator.line,
                            data.operator.lexeme
                        ),
                    }
                    .into());
                }
                let result = function(environment.clone(), arguments);
                return Ok(result);
            } else {
                return Err(LoxError::InterpretError {
                    message: format!(
                        "invalid function callable. line: {}. lexeme: {}",
                        data.operator.line, data.operator.lexeme
                    ),
                }
                .into());
            }
        }
        Expr::Get(_data) => Ok(LiteralValue::Nil),
        Expr::Grouping(data) => interpret_expr(environment.clone(), &data.expression),
        Expr::Literal(data) => Ok(data.value.clone()),
        Expr::Logical(data) => {
            let left = interpret_expr(environment.clone(), &data.left)?;
            match data.operator.typ {
                TokenType::Or => {
                    if left.is_truthy() {
                        return Ok(left);
                    }
                }
                TokenType::And => {
                    if !left.is_truthy() {
                        return Ok(left);
                    }
                }
                _ => {
                    return Err(LoxError::InterpretError {
                        message: format!(
                            "invalid logical op. line: {}. lexeme: {}",
                            data.operator.line, data.operator.lexeme
                        ),
                    }
                    .into());
                }
            }

            let right = interpret_expr(environment.clone(), &data.right)?;
            Ok(right)
        }
        Expr::Set(_data) => Ok(LiteralValue::Nil),
        Expr::Super(_data) => Ok(LiteralValue::Nil),
        Expr::This(_data) => Ok(LiteralValue::Nil),
        Expr::Unary(data) => {
            let right_value = interpret_expr(environment.clone(), &data.right)?;
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
            let value = environment.borrow().get(&data.name.lexeme);
            Ok(value)
        }
    }
}

fn interpret(environment: Rc<RefCell<Environment>>, stmt: &Stmt) -> anyhow::Result<()> {
    match stmt {
        Stmt::Block(block_stmt_data) => {
            let prev_env = environment.clone();
            let new_env = Rc::new(RefCell::new(Environment::new_with_parent(&prev_env)));
            {
                for stmt in &block_stmt_data.statements {
                    interpret(new_env.clone(), &stmt)?;
                }
            }
            Ok(())
        }
        Stmt::Class(_class_stmt_data) => Ok(()),
        Stmt::Expression(expression_stmt_data) => {
            interpret_expr(environment.clone(), &expression_stmt_data.expression)?;
            Ok(())
        }
        Stmt::Function(_function_stmt_data) => Ok(()),
        Stmt::If(if_stmt_data) => {
            let condition = interpret_expr(environment.clone(), &if_stmt_data.condition)?;
            if condition.is_truthy() {
                if let Some(then_branch) = &if_stmt_data.then_branch {
                    interpret(environment.clone(), then_branch)?;
                }
            } else {
                if let Some(else_branch) = &if_stmt_data.else_branch {
                    interpret(environment.clone(), else_branch)?;
                }
            }
            Ok(())
        }
        Stmt::Print(print_stmt_data) => {
            let v = interpret_expr(environment.clone(), &print_stmt_data.expression)?;
            println!("{}", v);
            Ok(())
        }
        Stmt::Return(_return_stmt_data) => Ok(()),
        Stmt::Variable(variable_stmt_data) => {
            let value = if let Some(initializer) = &variable_stmt_data.initializer {
                interpret_expr(environment.clone(), initializer)?
            } else {
                LiteralValue::Nil
            };
            environment
                .borrow_mut()
                .define(&variable_stmt_data.name.lexeme, value);

            Ok(())
        }
        Stmt::While(while_stmt_data) => {
            while interpret_expr(environment.clone(), &while_stmt_data.condition)?.is_truthy() {
                if let Some(stmt) = &while_stmt_data.body {
                    interpret(environment.clone(), stmt)?;
                }
            }
            Ok(())
        }
    }
}
