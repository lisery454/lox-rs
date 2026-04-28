use super::{
    expr::{Expr, VariableExprData},
    token::Token,
};

pub struct BlockStmtData {
    statements: Vec<Box<Stmt>>,
}

pub struct ClassStmtData {
    name: Token,
    superclass: VariableExprData,
    methods: Vec<FunctionStmtData>,
}

pub struct ExpressionStmtData {
    expression: Expr,
}

pub struct FunctionStmtData {
    name: Token,
    params: Vec<Token>,
    body: Vec<Box<Stmt>>,
}

pub struct IfStmtData {
    consition: Expr,
    then_branch: Box<Stmt>,
    else_branch: Box<Stmt>,
}

pub struct PrintStmtData {
    expression: Expr,
}

pub struct ReturnStmtData {
    keyword: Token,
    value: Expr,
}

pub struct VariableStmtData {
    name: Token,
    initializer: Expr,
}

pub struct WhileStmtData {
    condition: Expr,
    body: Box<Stmt>,
}

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
