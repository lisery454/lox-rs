use anyhow::Result;

use crate::{
    error::LoxError,
    model::{
        expr::{Expr, LiteralExprData, UnaryExprData, VariableExprData},
        literal::LiteralValue,
        stmt::{ExpressionStmtData, PrintStmtData, Stmt, VariableStmtData},
        token::{Token, TokenType},
    },
};

/// Tokens -> Expr
pub struct Parser {
    tokens: Vec<Token>,
    current: u64,
}

impl Parser {
    pub fn new(tokens: &Vec<Token>) -> Self {
        Self {
            tokens: tokens.clone(),
            current: 0,
        }
    }

    /// program -> stmt*
    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
        let mut stmts = Vec::new();

        while !self.is_at_end() {
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            }
        }

        Ok(stmts)
    }

    /// stmt -> declaration_stmt | statement_stmt
    fn parse_stmt(&mut self) -> Option<Stmt> {
        let result = if self.match_advance(|t| matches!(t, TokenType::Var)) {
            self.parse_var_declaration()
        } else {
            self.parse_statements()
        };

        if let Ok(stmt) = result {
            Some(stmt)
        } else {
            eprintln!("{:?}", result);
            self.synchronize();
            None
        }
    }

    /// declaration_stmt -> 'var' identifier '=' experssion
    fn parse_var_declaration(&mut self) -> Result<Stmt> {
        let identifier = self.consume(
            |t| matches!(t, TokenType::Identifier(_)),
            "Expect variable name.",
        )?;

        if let Some(identifier) = identifier {
            let name = identifier.clone();
            if self.match_advance(|t| matches!(t, TokenType::Equal)) {
                let initializer = self.parse_experssion()?;
                self.consume(
                    |t| matches!(t, TokenType::Semicolon),
                    "Expect ';' after variable declaration.",
                )?;
                return Ok(Stmt::Variable(VariableStmtData { name, initializer }));
            } else {
                return Err(LoxError::ParseError {
                    message: format!("Expect '=' after variable identifier."),
                }
                .into());
            }
        } else {
            return Err(LoxError::ParseError {
                message: format!("identifier is none."),
            }
            .into());
        }
    }

    /// statement_stmt -> print_stmt | expression_stmt
    fn parse_statements(&mut self) -> Result<Stmt> {
        if self.match_advance(|t| matches!(t, TokenType::Print)) {
            return self.parse_print_statements();
        }

        return self.parse_expression_statements();
    }

    /// print_stmt -> 'print' experssion ';'
    fn parse_print_statements(&mut self) -> Result<Stmt> {
        let expr = self.parse_experssion()?;
        self.consume(
            |t| matches!(t, TokenType::Semicolon),
            "Expect ';' after value (print).",
        )?;
        return Ok(Stmt::Print(PrintStmtData { expression: expr }));
    }

    /// expression_stmt -> experssion ';'
    fn parse_expression_statements(&mut self) -> Result<Stmt> {
        let expr = self.parse_experssion()?;
        self.consume(
            |t| matches!(t, TokenType::Semicolon),
            "Expect ';' after value (expr).",
        )?;
        return Ok(Stmt::Expression(ExpressionStmtData { expression: expr }));
    }

    /// experssion -> common_experssion
    fn parse_experssion(&mut self) -> Result<Expr> {
        self.parse_assignment()
    }

    /// common_experssion -> equality | assign
    /// assign -> equality '=' equality
    fn parse_assignment(&mut self) -> Result<Expr> {
        let expr = self.parse_equality()?;

        if self.match_advance(|t| matches!(t, TokenType::Equal)) {
            let prev_token = format!("{}", self.previous().unwrap());
            let value = self.parse_equality()?;
            if let Expr::Variable(v) = expr {
                let name = v.name;
                return Ok(Expr::assign(name, value));
            }
            return Err(LoxError::ParseError {
                message: format!("Invalid assignment target. at {:?}", prev_token),
            }
            .into());
        }

        return Ok(expr);
    }

    /// equality -> comparison (('!=' | '==') comparison)*
    fn parse_equality(&mut self) -> Result<Expr> {
        let mut expr: Expr = self.parse_comparison()?;

        while self.match_advance(|t| matches!(t, TokenType::BangEqual | TokenType::EqualEqual)) {
            let operator = (*self.previous().unwrap()).clone();
            let right = self.parse_comparison()?;
            expr = Expr::binary(operator, expr, right);
        }

        Ok(expr)
    }

    /// comparison -> term (('>' | '>=' | '<' | '<=') term)*
    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut expr = self.parse_term()?;

        while self.match_advance(|t| {
            matches!(
                t,
                TokenType::Greater
                    | TokenType::GreaterEqual
                    | TokenType::Less
                    | TokenType::LessEqual
            )
        }) {
            let operator = (*self.previous().unwrap()).clone();
            let right = self.parse_term()?;
            expr = Expr::binary(operator, expr, right);
        }

        Ok(expr)
    }

    /// term -> factor (('+' | '-') factor)*
    fn parse_term(&mut self) -> Result<Expr> {
        let mut expr = self.parse_factor()?;

        while self.match_advance(|t| matches!(t, TokenType::Plus | TokenType::Minus)) {
            let operator = (*self.previous().unwrap()).clone();
            let right = self.parse_factor()?;
            expr = Expr::binary(operator, expr, right);
        }

        Ok(expr)
    }

    /// factor -> unary (('*' | '/') unary)*
    fn parse_factor(&mut self) -> Result<Expr> {
        let mut expr = self.parse_unary()?;

        while self.match_advance(|t| matches!(t, TokenType::Slash | TokenType::Star)) {
            let operator = (*self.previous().unwrap()).clone();
            let right = self.parse_unary()?;
            expr = Expr::binary(operator, expr, right);
        }

        Ok(expr)
    }

    /// unary -> ('-' | '!') unary
    ///            | primary
    fn parse_unary(&mut self) -> Result<Expr> {
        if self.match_advance(|t| matches!(t, TokenType::Minus | TokenType::Bang)) {
            let operator = (*self.previous().unwrap()).clone();
            let right = self.parse_unary()?;
            return Ok(Expr::unary(operator, right));
        }

        self.parse_primary()
    }

    /// primary -> false | true | nil | number | string | identifier
    ///             |  ( experssion )
    fn parse_primary(&mut self) -> Result<Expr> {
        if self.match_advance(|t| matches!(t, TokenType::False)) {
            return Ok(Expr::literal(false));
        }

        if self.match_advance(|t| matches!(t, TokenType::True)) {
            return Ok(Expr::literal(true));
        }

        if self.match_advance(|t| matches!(t, TokenType::Nil)) {
            return Ok(Expr::literal(()));
        }

        if let Some(n) = self.match_some_advance(|t| match t {
            TokenType::Number(n) => Some(n.clone()),
            _ => None,
        }) {
            return Ok(Expr::literal(n));
        }

        if let Some(n) = self.match_some_advance(|t| match t {
            TokenType::String(n) => Some(n.clone()),
            _ => None,
        }) {
            return Ok(Expr::literal(n));
        }

        if self.match_advance(|t| matches!(t, TokenType::Identifier(_))) {
            return Ok(Expr::variable(self.previous().unwrap().clone()));
        }

        if self.match_advance(|t| matches!(t, TokenType::LeftParen)) {
            let expr = self.parse_experssion()?;
            self.consume(
                |t| matches!(t, TokenType::RightBrace),
                "not find right paren <)>",
            )?;
            return Ok(Expr::grouping(expr));
        }

        return Err(LoxError::ParseError {
            message: format!("Expect expression. at {:?}", self.previous()),
        }
        .into());
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if let Some(t) = self.previous()
                && matches!(t.typ, TokenType::Semicolon)
            {
                return;
            }

            if let Some(t) = self.peek()
                && matches!(
                    t.typ,
                    TokenType::Class
                        | TokenType::Function
                        | TokenType::Var
                        | TokenType::For
                        | TokenType::If
                        | TokenType::While
                        | TokenType::Print
                        | TokenType::Return
                )
            {
                return;
            }

            self.advance();
        }
    }

    fn consume<F>(&mut self, f: F, message: &str) -> Result<Option<&Token>>
    where
        F: FnOnce(&TokenType) -> bool,
    {
        if let Some(t) = self.peek()
            && f(&t.typ)
        {
            return Ok(self.advance());
        }

        let message = if let Some(t) = self.previous() {
            format!("{} at line {}", message, t.line)
        } else {
            format!("{}", message)
        };
        return Err(LoxError::ParseError { message }.into());
    }

    fn match_advance<F>(&mut self, f: F) -> bool
    where
        F: FnOnce(&TokenType) -> bool,
    {
        if let Some(token) = self.peek()
            && f(&token.typ)
        {
            self.advance();
            return true;
        } else {
            return false;
        }
    }

    fn match_some_advance<F, T>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&TokenType) -> Option<T>,
    {
        if let Some(token) = self.peek()
            && let Some(t) = f(&token.typ)
        {
            self.advance();
            return Some(t);
        } else {
            return None;
        }
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        return self.previous();
    }

    fn is_at_end(&self) -> bool {
        if let Some(token) = self.peek()
            && token.typ == TokenType::Eof
        {
            return true;
        }
        false
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current as usize)
    }

    /// get ref of token before current position. if current position is 0, return None
    fn previous(&self) -> Option<&Token> {
        if self.current == 0 {
            return None;
        }
        self.tokens.get((self.current - 1) as usize)
    }
}
