use std::{cell::RefCell, collections::HashMap, rc::Rc};

use chrono::{DateTime, Local};

use crate::{
    error::{LoxError, LoxResult},
    model::{
        environment::Environment,
        expr::Expr,
        literal::{LiteralValue, LoxClass, LoxFunction, LoxInstance},
        stmt::{FunctionStmtData, Stmt},
        token::{Token, TokenType},
    },
};

#[derive(Debug, Clone)]
enum ClassType {
    None,
    Class,
}

#[derive(Debug, Clone)]
enum FunctionType {
    None,
    Function,
    Method,
}

struct InterpreterContext {
    curr_env: Rc<RefCell<Environment>>,
    global_env: Rc<RefCell<Environment>>,
    locals: Rc<RefCell<HashMap<Token, u32>>>,
}

impl InterpreterContext {
    pub fn new() -> Self {
        let environment = Rc::new(RefCell::new(Environment::new()));

        let result = Self {
            curr_env: Rc::clone(&environment),
            global_env: Rc::clone(&environment),
            locals: Rc::new(RefCell::new(HashMap::new())),
        };

        result.global_env.borrow_mut().define(
            &"clock".to_string(),
            LiteralValue::Callable(Rc::new(LoxFunction::new(
                Rc::new(|_env, _args| {
                    let now: DateTime<Local> = Local::now();
                    Ok(LiteralValue::String(
                        now.format("%Y-%m-%d %H:%M:%S").to_string(),
                    ))
                }),
                0,
                Rc::clone(&result.curr_env),
            ))),
        );

        result
    }

    pub fn clone(&self) -> Self {
        Self {
            curr_env: Rc::clone(&self.curr_env),
            global_env: Rc::clone(&self.global_env),
            locals: Rc::clone(&self.locals),
        }
    }

    pub fn clone_with_new_env(&self, new_env: Rc<RefCell<Environment>>) -> Self {
        Self {
            curr_env: new_env,
            global_env: Rc::clone(&self.global_env),
            locals: Rc::clone(&self.locals),
        }
    }
}

pub struct Interpreter {
    context: InterpreterContext,
    scopes: Vec<HashMap<String, bool>>,
    function_type: FunctionType,
    class_type: ClassType,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            context: InterpreterContext::new(),
            scopes: Vec::new(),
            function_type: FunctionType::None,
            class_type: ClassType::None,
        }
    }

    pub fn interpret(&mut self, stmts: &Vec<Stmt>) -> LoxResult<()> {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }

        for stmt in stmts {
            interpret_stmt(self.context.clone(), stmt)?;
        }
        Ok(())
    }
}

fn interpret_expr(context: InterpreterContext, expr: &Expr) -> LoxResult<LiteralValue> {
    match expr {
        Expr::Assign(data) => {
            let value = interpret_expr(context.clone(), &data.value)?;
            let success = if let Some(distance) = context.locals.borrow().get(&data.name) {
                context
                    .curr_env
                    .borrow_mut()
                    .assign_at(distance, &data.name.lexeme, value)
            } else {
                context
                    .global_env
                    .borrow_mut()
                    .assign(&data.name.lexeme, value)
            };
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
            let left_value = interpret_expr(context.clone(), &data.left)?;
            let right_value = interpret_expr(context.clone(), &data.right)?;
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
            let callable = interpret_expr(context.clone(), &data.callee)?;
            let mut arguments = Vec::new();
            for argument in &data.arguments {
                let arg = interpret_expr(context.clone(), &argument)?;
                arguments.push(arg);
            }
            if let LiteralValue::Callable(func) = callable {
                if arguments.len() != func.arg_size {
                    return Err(LoxError::InterpretError {
                        message: format!(
                            "Expected {} arguments but got {}. line: {}. lexeme: {}",
                            func.arg_size,
                            arguments.len(),
                            data.operator.line,
                            data.operator.lexeme
                        ),
                    }
                    .into());
                }
                let result = (func.function)(Rc::clone(&func.closure), arguments)?;
                return Ok(result);
            } else if let LiteralValue::Class(class) = callable {
                let size = class.cons_len();
                if arguments.len() != size {
                    return Err(LoxError::InterpretError {
                        message: format!(
                            "Expected {} arguments but got {}. line: {}. lexeme: {}",
                            size,
                            arguments.len(),
                            data.operator.line,
                            data.operator.lexeme
                        ),
                    }
                    .into());
                }
                let instance = Rc::new(LoxInstance::new(&class));
                if let Some(constructor) = &instance.class.constructor {
                    let this = LiteralValue::ClassInstance(Rc::clone(&instance));
                    let func = constructor.clone().bind(this);
                    (func.function)(Rc::clone(&func.closure), arguments)?;
                }
                let value = LiteralValue::ClassInstance(instance);

                return Ok(value);
            } else {
                return Err(LoxError::InterpretError {
                    message: format!(
                        "invalid function callable. line: {}. lexeme: {}. callable: {}",
                        data.operator.line, data.operator.lexeme, callable
                    ),
                }
                .into());
            }
        }
        Expr::Get(data) => {
            let value = interpret_expr(context.clone(), &data.object)?;
            if let LiteralValue::ClassInstance(instance) = value {
                return Ok(instance.get(&data.name));
            } else {
                return Err(LoxError::InterpretError {
                    message: format!(
                        "Only instances have properties. line: {}. lexeme: {}",
                        data.name.line, data.name.lexeme
                    ),
                }
                .into());
            }
        }
        Expr::Grouping(data) => interpret_expr(context.clone(), &data.expression),
        Expr::Literal(data) => Ok(data.value.clone()),
        Expr::Logical(data) => {
            let left = interpret_expr(context.clone(), &data.left)?;
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

            let right = interpret_expr(context.clone(), &data.right)?;
            Ok(right)
        }
        Expr::Set(data) => {
            let obj = interpret_expr(context.clone(), &data.object)?;
            if let LiteralValue::ClassInstance(instance) = obj {
                let val = interpret_expr(context.clone(), &data.value)?;
                instance.set(&data.name, val.clone());
                return Ok(val);
            } else {
                return Err(LoxError::InterpretError {
                    message: format!(
                        "Only instances have fields. line: {}. lexeme: {}",
                        data.name.line, data.name.lexeme
                    ),
                }
                .into());
            }
        }
        Expr::Super(_data) => Ok(LiteralValue::Nil),
        Expr::This(data) => {
            if let Some(distance) = context.locals.borrow().get(&data.keyword) {
                Ok(context
                    .curr_env
                    .borrow()
                    .get_at(distance, &data.keyword.lexeme))
            } else {
                Ok(context.global_env.borrow().get(&data.keyword.lexeme))
            }
        }
        Expr::Unary(data) => {
            let right_value = interpret_expr(context.clone(), &data.right)?;
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
            if let Some(distance) = context.locals.borrow().get(&data.name) {
                Ok(context
                    .curr_env
                    .borrow()
                    .get_at(distance, &data.name.lexeme))
            } else {
                Ok(context.global_env.borrow().get(&data.name.lexeme))
            }
        }
    }
}

fn interpret_stmt(context: InterpreterContext, stmt: &Stmt) -> LoxResult<()> {
    match stmt {
        Stmt::Block(block_stmt_data) => {
            let prev_env = Rc::clone(&context.curr_env);
            let new_env = Rc::new(RefCell::new(Environment::new_with_parent(&prev_env)));

            for stmt in &block_stmt_data.statements {
                interpret_stmt(context.clone_with_new_env(Rc::clone(&new_env)), &stmt)?;
            }

            Ok(())
        }
        Stmt::Class(class_stmt_data) => {
            context
                .curr_env
                .borrow_mut()
                .define(&class_stmt_data.name.lexeme, LiteralValue::Nil);

            let mut methods = HashMap::new();
            for method in &class_stmt_data.methods {
                if let Stmt::Function(func_data) = &**method {
                    let function = create_function(func_data, context.clone());
                    methods.insert(func_data.name.lexeme.clone(), function);
                }
            }

            context.curr_env.borrow_mut().assign(
                &class_stmt_data.name.lexeme,
                LiteralValue::Class(Rc::new(LoxClass::new(
                    &class_stmt_data.name.lexeme,
                    methods,
                ))),
            );
            Ok(())
        }
        Stmt::Expression(expression_stmt_data) => {
            interpret_expr(context.clone(), &expression_stmt_data.expression)?;
            Ok(())
        }
        Stmt::Function(function_stmt_data) => {
            let name = function_stmt_data.name.lexeme.clone();

            context.curr_env.borrow_mut().define(
                &name,
                LiteralValue::Callable(create_function(function_stmt_data, context.clone())),
            );

            Ok(())
        }
        Stmt::If(if_stmt_data) => {
            let condition = interpret_expr(context.clone(), &if_stmt_data.condition)?;
            if condition.is_truthy() {
                if let Some(then_branch) = &if_stmt_data.then_branch {
                    interpret_stmt(context.clone(), then_branch)?;
                }
            } else {
                if let Some(else_branch) = &if_stmt_data.else_branch {
                    interpret_stmt(context.clone(), else_branch)?;
                }
            }
            Ok(())
        }
        Stmt::Print(print_stmt_data) => {
            let v = interpret_expr(context.clone(), &print_stmt_data.expression)?;
            println!("{}", v);
            Ok(())
        }
        Stmt::Return(return_stmt_data) => match &return_stmt_data.value {
            Some(value) => Err(LoxError::ReturnError(interpret_expr(
                context.clone(),
                value,
            )?)),
            None => Err(LoxError::ReturnError(LiteralValue::Nil)),
        },
        Stmt::Variable(variable_stmt_data) => {
            let value = if let Some(initializer) = &variable_stmt_data.initializer {
                interpret_expr(context.clone(), initializer)?
            } else {
                LiteralValue::Nil
            };
            context
                .curr_env
                .borrow_mut()
                .define(&variable_stmt_data.name.lexeme, value);

            Ok(())
        }
        Stmt::While(while_stmt_data) => {
            while interpret_expr(context.clone(), &while_stmt_data.condition)?.is_truthy() {
                if let Some(stmt) = &while_stmt_data.body {
                    interpret_stmt(context.clone(), stmt)?;
                }
            }
            Ok(())
        }
    }
}

fn create_function(
    function_stmt_data: &FunctionStmtData,
    context: InterpreterContext,
) -> Rc<LoxFunction> {
    let body = function_stmt_data.body.clone();
    let func_params = function_stmt_data.params.clone();
    let context_move = context.clone();

    return Rc::new(LoxFunction::new(
        Rc::new(move |env, params| {
            let env = Rc::new(RefCell::new(Environment::new_with_parent(&env)));
            for i in 0..func_params.len() {
                let name = func_params[i].lexeme.clone();
                let value = params[i].clone();
                env.borrow_mut().define(&name, value);
            }
            let result = interpret_stmt(context_move.clone_with_new_env(env), &body);
            match result {
                Ok(_) => Ok(LiteralValue::Nil),
                Err(LoxError::ReturnError(v)) => Ok(v),
                Err(_) => Ok(LiteralValue::Nil),
            }
        }),
        function_stmt_data.params.len(),
        Rc::clone(&context.curr_env),
    ));
}

// resolver
impl Interpreter {
    fn resolve_stmt(&mut self, stmt: &Stmt) -> LoxResult<()> {
        match stmt {
            Stmt::Block(block_stmt_data) => {
                self.begin_scope();
                for stmt in &block_stmt_data.statements {
                    self.resolve_stmt(stmt)?;
                }
                self.end_scope();
                Ok(())
            }
            Stmt::Class(class_stmt_data) => {
                let enclosing_class_type = self.class_type.clone();
                self.class_type = ClassType::Class;

                self.declare(&class_stmt_data.name)?;
                self.define(&class_stmt_data.name);

                self.begin_scope();

                self.scopes.last_mut().unwrap().insert("this".into(), true);

                for method in &class_stmt_data.methods {
                    if let Stmt::Function(func_data) = &**method {
                        self.resolve_function(func_data, FunctionType::Method)?;
                    }
                }

                self.end_scope();

                self.class_type = enclosing_class_type;

                Ok(())
            }
            Stmt::Expression(expression_stmt_data) => {
                self.resolve_expr(&expression_stmt_data.expression)?;
                Ok(())
            }
            Stmt::Function(function_stmt_data) => {
                self.declare(&function_stmt_data.name)?;
                self.define(&function_stmt_data.name);
                self.resolve_function(&function_stmt_data, FunctionType::Function)?;
                Ok(())
            }
            Stmt::If(if_stmt_data) => {
                self.resolve_expr(&if_stmt_data.condition)?;
                if let Some(then_branch) = &if_stmt_data.then_branch {
                    self.resolve_stmt(then_branch)?;
                }
                if let Some(else_branch) = &if_stmt_data.else_branch {
                    self.resolve_stmt(else_branch)?;
                }
                Ok(())
            }
            Stmt::Print(print_stmt_data) => {
                self.resolve_expr(&print_stmt_data.expression)?;
                Ok(())
            }
            Stmt::Return(return_stmt_data) => {
                if matches!(self.function_type, FunctionType::None) {
                    return Err(LoxError::InterpretError {
                        message: "Can't return from top-level code.".into(),
                    }
                    .into());
                }
                if let Some(value) = &return_stmt_data.value {
                    self.resolve_expr(value)?;
                }
                Ok(())
            }
            Stmt::Variable(variable_stmt_data) => {
                self.declare(&variable_stmt_data.name)?;
                if let Some(initializer) = &variable_stmt_data.initializer {
                    self.resolve_expr(initializer)?;
                }
                self.define(&variable_stmt_data.name);
                Ok(())
            }
            Stmt::While(while_stmt_data) => {
                self.resolve_expr(&while_stmt_data.condition)?;
                if let Some(body) = &while_stmt_data.body {
                    self.resolve_stmt(body)?;
                }
                Ok(())
            }
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) -> LoxResult<()> {
        match expr {
            Expr::Assign(assign_expr_data) => {
                self.resolve_expr(&assign_expr_data.value)?;
                self.resolve_local_var(&assign_expr_data.name);
                Ok(())
            }
            Expr::Binary(binary_expr_data) => {
                self.resolve_expr(&binary_expr_data.left)?;
                self.resolve_expr(&binary_expr_data.right)?;
                Ok(())
            }
            Expr::Call(call_expr_data) => {
                self.resolve_expr(&call_expr_data.callee)?;
                for expr in &call_expr_data.arguments {
                    self.resolve_expr(expr)?;
                }
                Ok(())
            }
            Expr::Get(get_expr_data) => {
                self.resolve_expr(&get_expr_data.object)?;
                Ok(())
            }
            Expr::Grouping(grouping_expr_data) => {
                self.resolve_expr(&grouping_expr_data.expression)?;
                Ok(())
            }
            Expr::Literal(_literal_expr_data) => Ok(()),
            Expr::Logical(logical_expr_data) => {
                self.resolve_expr(&logical_expr_data.left)?;
                self.resolve_expr(&logical_expr_data.right)?;
                Ok(())
            }
            Expr::Set(set_expr_data) => {
                self.resolve_expr(&set_expr_data.value)?;
                self.resolve_expr(&set_expr_data.object)?;
                Ok(())
            }
            Expr::Super(_super_expr_data) => Ok(()),
            Expr::This(this_expr_data) => {
                if matches!(self.class_type, ClassType::None) {
                    return Err(LoxError::InterpretError {
                        message: "Can't use 'this' outside of a class.".into(),
                    }
                    .into());
                }

                self.resolve_local_var(&this_expr_data.keyword);

                Ok(())
            }
            Expr::Unary(unary_expr_data) => {
                self.resolve_expr(&unary_expr_data.right)?;
                Ok(())
            }
            Expr::Variable(variable_expr_data) => {
                if !self.scopes.is_empty()
                    && let Some(scope) = self.scopes.last()
                    && let Some(is_defined) = scope.get(&variable_expr_data.name.lexeme)
                    && *is_defined == false
                {
                    return Err(LoxError::InterpretError {
                        message: "Can't read local variable in its own initializer.".into(),
                    }
                    .into());
                }

                self.resolve_local_var(&variable_expr_data.name);

                Ok(())
            }
        }
    }

    fn resolve_local_var(&mut self, token: &Token) {
        for scope_i in (0..self.scopes.len()).rev() {
            let scope = &self.scopes[scope_i];
            if scope.contains_key(&token.lexeme) {
                self.interpreter_resolve(token, (self.scopes.len() - 1 - scope_i) as u32);
                return;
            }
        }
    }

    fn interpreter_resolve(&mut self, token: &Token, depth: u32) {
        self.context
            .locals
            .borrow_mut()
            .insert(token.clone(), depth);
    }

    fn resolve_function(
        &mut self,
        func_data: &FunctionStmtData,
        function_type: FunctionType,
    ) -> LoxResult<()> {
        self.begin_scope();

        let enclosing_function_type = self.function_type.clone();
        self.function_type = function_type;

        for token in &func_data.params {
            self.declare(token)?;
            self.define(token);
        }

        self.resolve_stmt(&func_data.body)?;

        self.end_scope();

        self.function_type = enclosing_function_type;
        Ok(())
    }

    fn declare(&mut self, token: &Token) -> LoxResult<()> {
        if let Some(scope) = self.scopes.last_mut() {
            let name = token.lexeme.clone();
            if scope.contains_key(&name) {
                return Err(LoxError::InterpretError {
                    message: format!(
                        "Already a variable with this name <{}> in this scope.",
                        name
                    ),
                }
                .into());
            }
            scope.insert(token.lexeme.clone(), false);
        }

        Ok(())
    }

    fn define(&mut self, token: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(token.lexeme.clone(), true);
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }
}
