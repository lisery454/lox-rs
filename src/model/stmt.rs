use super::{
    expr::{Expr, VariableExprData},
    token::Token,
};

#[derive(Debug, Clone)]
pub struct BlockStmtData {
    pub(crate) statements: Vec<Box<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct ClassStmtData {
    pub(crate) name: Token,
    pub(crate) superclass: VariableExprData,
    pub(crate) methods: Vec<FunctionStmtData>,
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
    pub(crate) consition: Expr,
    pub(crate) then_branch: Box<Stmt>,
    pub(crate) else_branch: Box<Stmt>,
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
    pub(crate) initializer: Expr,
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
