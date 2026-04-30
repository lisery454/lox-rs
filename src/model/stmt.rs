use anyhow::bail;

use super::{expr::Expr, token::Token};

#[derive(Debug, Clone)]
pub struct BlockStmtData {
    pub(crate) statements: Vec<Box<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct ClassStmtData {
    pub(crate) name: Token,
    pub(crate) superclass: Expr,        // variable
    pub(crate) methods: Vec<Box<Stmt>>, // func
}

#[derive(Debug, Clone)]
pub struct ExpressionStmtData {
    pub(crate) expression: Expr,
}

#[derive(Debug, Clone)]
pub struct FunctionStmtData {
    pub(crate) name: Token,
    pub(crate) params: Vec<Token>,
    pub(crate) body: Vec<Box<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct IfStmtData {
    pub(crate) condition: Expr,
    pub(crate) then_branch: Option<Box<Stmt>>,
    pub(crate) else_branch: Option<Box<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct PrintStmtData {
    pub(crate) expression: Expr,
}

#[derive(Debug, Clone)]
pub struct ReturnStmtData {
    pub(crate) keyword: Token,
    pub(crate) value: Expr,
}

#[derive(Debug, Clone)]
pub struct VariableStmtData {
    pub(crate) name: Token,
    pub(crate) initializer: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct WhileStmtData {
    pub(crate) condition: Expr,
    pub(crate) body: Box<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(BlockStmtData),
    Class(ClassStmtData),
    Expression(ExpressionStmtData),
    Function(FunctionStmtData),
    If(IfStmtData),
    Print(PrintStmtData),
    Return(ReturnStmtData),
    Variable(VariableStmtData),
    While(WhileStmtData),
}

impl Stmt {
    pub fn block(stmts: Vec<Stmt>) -> Stmt {
        Stmt::Block(BlockStmtData {
            statements: stmts.into_iter().map(|s| Box::new(s)).collect(),
        })
    }

    pub fn class(class_name: Token, super_class: Expr, methods: Vec<Stmt>) -> Stmt {
        Stmt::Class(ClassStmtData {
            name: class_name,
            superclass: super_class,
            methods: methods.into_iter().map(|e| Box::new(e)).collect(),
        })
    }

    pub fn expression(expression: Expr) -> Stmt {
        Stmt::Expression(ExpressionStmtData { expression })
    }

    pub fn function(name: Token, params: Vec<Token>, body: Vec<Stmt>) -> Stmt {
        Stmt::Function(FunctionStmtData {
            name,
            params,
            body: body.into_iter().map(|e| Box::new(e)).collect(),
        })
    }

    pub fn if_(condition: Expr, then_branch: Option<Stmt>, else_branch: Option<Stmt>) -> Stmt {
        Stmt::If(IfStmtData {
            condition,
            then_branch: then_branch.map(|b| Box::new(b)),
            else_branch: else_branch.map(|b| Box::new(b)),
        })
    }

    pub fn print(expression: Expr) -> Stmt {
        Stmt::Print(PrintStmtData { expression })
    }

    pub fn return_(keyword: Token, value: Expr) -> Stmt {
        Stmt::Return(ReturnStmtData { keyword, value })
    }

    pub fn variable(name: Token, initializer: Option<Expr>) -> Stmt {
        Stmt::Variable(VariableStmtData { name, initializer })
    }

    pub fn while_(condition: Expr, body: Stmt) -> Stmt {
        Stmt::While(WhileStmtData {
            condition,
            body: Box::new(body),
        })
    }
}
