use std::mem::discriminant;

use anyhow::Result;

use crate::{
    error::LoxError,
    model::{
        expr::Expr,
        stmt::Stmt,
        token::{Token, TokenType},
    },
};

/// Tokens -> Expr -> Stmt
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

    /// stmt -> var_declaration_stmt | statement_stmt | fun_declaration_stmt
    fn parse_stmt(&mut self) -> Option<Stmt> {
        let result = if self.match_advance(|t| matches!(t, TokenType::Var)) {
            self.parse_var_declaration()
        } else if self.match_advance(|t| matches!(t, TokenType::Function)) {
            self.parse_fun_declaration("function")
        } else {
            self.parse_statement()
        };

        if let Ok(stmt) = result {
            Some(stmt)
        } else {
            eprintln!("{:?}", result);
            self.synchronize();
            None
        }
    }

    /// fun_declaration_stmt -> 'fun' identifier '(' parameters? ')' block
    /// parameters     → identifier ( ',' identifier )* ;
    fn parse_fun_declaration(&mut self, kind: &str) -> Result<Stmt> {
        let name = self
            .consume(|t| matches!(t, TokenType::Identifier(_)), "")?
            .clone();

        self.consume(
            |t| matches!(t, TokenType::LeftParen),
            format!("Expect '(' after {} name.", kind),
        )?;

        let mut params = Vec::new();

        if !self.check(TokenType::RightParen) {
            loop {
                if params.len() > 255 {
                    return Err(LoxError::ParseError {
                        message: "Can't have more than 255 parameters.".to_string(),
                    }
                    .into());
                }

                params.push(
                    self.consume(
                        |t| matches!(t, TokenType::Identifier(_)),
                        "Expect parameter name.",
                    )?
                    .clone(),
                );

                if !self.match_advance(|t| matches!(t, TokenType::Comma)) {
                    break;
                }
            }
        }

        self.consume(
            |t| matches!(t, TokenType::RightParen),
            format!("Expect ')' after parameters."),
        )?;

        self.consume(
            |t| matches!(t, TokenType::LeftBrace),
            format!("Expect '{{' before {} body.", kind),
        )?;

        let body = self.parse_block_statements()?;
        Ok(Stmt::function(name, params, body))
    }

    /// var_declaration_stmt -> 'var' identifier '=' experssion
    fn parse_var_declaration(&mut self) -> Result<Stmt> {
        let identifier = self.consume(
            |t| matches!(t, TokenType::Identifier(_)),
            "Expect variable name.",
        )?;

        let name = identifier.clone();
        if self.match_advance(|t| matches!(t, TokenType::Equal)) {
            let initializer = self.parse_experssion()?;
            self.consume(
                |t| matches!(t, TokenType::Semicolon),
                "Expect ';' after variable declaration.",
            )?;
            return Ok(Stmt::variable(name, Some(initializer)));
        } else {
            self.consume(
                |t| matches!(t, TokenType::Semicolon),
                "Expect ';' after variable declaration.",
            )?;
            return Ok(Stmt::variable(name, None));
        }
    }

    /// statement_stmt -> print_stmt | expression_stmt | block_stmt | if_stmt | while_stmt | for_stmt
    fn parse_statement(&mut self) -> Result<Stmt> {
        if self.match_advance(|t| matches!(t, TokenType::Print)) {
            return self.parse_print_statements();
        } else if self.match_advance(|t| matches!(t, TokenType::LeftBrace)) {
            return self.parse_block_statements();
        } else if self.match_advance(|t| matches!(t, TokenType::While)) {
            return self.parse_while_statements();
        } else if self.match_advance(|t| matches!(t, TokenType::If)) {
            return self.parse_if_statements();
        } else if self.match_advance(|t| matches!(t, TokenType::For)) {
            return self.parse_for_statements();
        } else {
            return self.parse_expression_statement();
        }
    }

    // for_stmt -> 'for' '('  ')'
    fn parse_for_statements(&mut self) -> Result<Stmt> {
        self.consume(
            |t| matches!(t, TokenType::LeftParen),
            "Expect '(' after 'for'.",
        )?;

        let initializer = if self.match_advance(|t| matches!(t, TokenType::Semicolon)) {
            None
        } else if self.match_advance(|t| matches!(t, TokenType::Var)) {
            Some(self.parse_var_declaration()?)
        } else {
            Some(self.parse_expression_statement()?)
        };

        let condition = if !self.check(TokenType::Semicolon) {
            Some(self.parse_experssion()?)
        } else {
            None
        };
        self.consume(
            |t| matches!(t, TokenType::Semicolon),
            "Expect ';' after for loop condition.",
        )?;

        let increment = if !self.check(TokenType::RightParen) {
            Some(self.parse_experssion()?)
        } else {
            None
        };
        self.consume(
            |t| matches!(t, TokenType::RightParen),
            "Expect ')' after for loop increment.",
        )?;

        let mut body = self.parse_statement()?;

        if let Some(increment) = increment {
            body = Stmt::block(vec![body, Stmt::expression(increment)]);
        }

        if let Some(condition) = condition {
            body = Stmt::while_(condition, Some(body))
        } else {
            body = Stmt::while_(Expr::literal(true), Some(body))
        }

        if let Some(initializer) = initializer {
            body = Stmt::block(vec![initializer, body]);
        }
        Ok(body)
    }

    // while_stmt -> 'while' '(' expr ')'  stmt
    fn parse_while_statements(&mut self) -> Result<Stmt> {
        self.consume(
            |t| matches!(t, TokenType::LeftParen),
            "Expect '(' after 'while'.",
        )?;

        let condition = self.parse_experssion()?;

        self.consume(
            |t| matches!(t, TokenType::RightParen),
            "Expect ')' after 'while' condition.",
        )?;

        let body = self.parse_stmt();

        Ok(Stmt::while_(condition, body))
    }

    // if_stmt -> 'if' '(' expr ')'  stmt  ('else' stmt)?
    fn parse_if_statements(&mut self) -> Result<Stmt> {
        self.consume(
            |t| matches!(t, TokenType::LeftParen),
            "Expect '(' after 'if'.",
        )?;

        let condition = self.parse_experssion()?;

        self.consume(
            |t| matches!(t, TokenType::RightParen),
            "Expect ')' after 'if' condition.",
        )?;

        let then_branch = self.parse_stmt();
        let mut else_branch = None;
        if self.match_advance(|t| matches!(t, TokenType::Else)) {
            else_branch = self.parse_stmt();
        }
        Ok(Stmt::if_(condition, then_branch, else_branch))
    }

    // block_stmt -> '{' stmt '}'
    fn parse_block_statements(&mut self) -> Result<Stmt> {
        let mut stmts = Vec::new();

        while !self.is_at_end() && !self.check(TokenType::RightBrace) {
            if let Some(s) = self.parse_stmt() {
                stmts.push(s);
            }
        }
        self.consume(
            |t| matches!(t, TokenType::RightBrace),
            "Expect '}' after block.",
        )?;
        Ok(Stmt::block(stmts))
    }

    /// print_stmt -> 'print' experssion ';'
    fn parse_print_statements(&mut self) -> Result<Stmt> {
        let expr = self.parse_experssion()?;
        self.consume(
            |t| matches!(t, TokenType::Semicolon),
            "Expect ';' after value (print).",
        )?;
        return Ok(Stmt::print(expr));
    }

    /// expression_stmt -> experssion ';'
    fn parse_expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.parse_experssion()?;
        self.consume(
            |t| matches!(t, TokenType::Semicolon),
            "Expect ';' after value (expr).",
        )?;
        return Ok(Stmt::expression(expr));
    }

    /// experssion -> common_experssion
    fn parse_experssion(&mut self) -> Result<Expr> {
        self.parse_assignment()
    }

    /// common_experssion -> logic_or | assign
    /// assign -> logic_or '=' logic_or
    fn parse_assignment(&mut self) -> Result<Expr> {
        let expr = self.parse_logic_or()?;

        if self.match_advance(|t| matches!(t, TokenType::Equal)) {
            let prev_token = format!("{}", self.previous().unwrap());
            let value = self.parse_logic_or()?;
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

    /// logic_or -> logic_and ('or' logic_and)*
    fn parse_logic_or(&mut self) -> Result<Expr> {
        let mut expr = self.parse_logic_and()?;
        if self.match_advance(|t| matches!(t, TokenType::Or)) {
            let prev_token = self.previous().unwrap().clone();
            let right = self.parse_logic_and()?;
            expr = Expr::logical(prev_token, expr, right);
        }
        return Ok(expr);
    }

    /// logic_and -> equality ('and' equality)*
    fn parse_logic_and(&mut self) -> Result<Expr> {
        let mut expr = self.parse_equality()?;
        if self.match_advance(|t| matches!(t, TokenType::And)) {
            let prev_token = self.previous().unwrap().clone();
            let right = self.parse_equality()?;
            expr = Expr::logical(prev_token, expr, right);
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
    ///            | call
    fn parse_unary(&mut self) -> Result<Expr> {
        if self.match_advance(|t| matches!(t, TokenType::Minus | TokenType::Bang)) {
            let operator = (*self.previous().unwrap()).clone();
            let right = self.parse_unary()?;
            return Ok(Expr::unary(operator, right));
        }

        self.parse_call()
    }

    /// call -> primary ( '('  arguments ')' )*
    /// arguments -> expression (',' expression)*
    fn parse_call(&mut self) -> Result<Expr> {
        let mut callee = self.parse_primary()?;
        while self.match_advance(|t| matches!(t, TokenType::LeftParen)) {
            let mut arguments = Vec::new();
            if !self.check(TokenType::RightParen) {
                loop {
                    arguments.push(self.parse_experssion()?);

                    if !self.match_advance(|t| matches!(t, TokenType::Comma)) {
                        break;
                    }
                }
            }

            let paren = self.consume(
                |t| matches!(t, TokenType::RightParen),
                "Expect ')' after arguments.",
            )?;

            if arguments.len() >= 255 {
                return Err(LoxError::ParseError {
                    message: format!("Can't have more than 255 arguments. at line {}", paren.line),
                }
                .into());
            }

            callee = Expr::call(paren.clone(), callee, arguments)
        }
        return Ok(callee);
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

    fn consume<F, S>(&mut self, f: F, message: S) -> Result<&Token>
    where
        F: FnOnce(&TokenType) -> bool,
        S: AsRef<str>,
    {
        if let Some(t) = self.peek()
            && f(&t.typ)
        {
            return Ok(self.advance());
        }

        let message = if let Some(t) = self.previous() {
            format!("{} at line {}", message.as_ref(), t.line)
        } else {
            format!("{}", message.as_ref())
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

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        return self.previous().unwrap();
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

    fn check(&self, typ: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        if let Some(t) = self.peek()
            && discriminant(&t.typ) == discriminant(&typ)
        {
            return true;
        }
        return false;
    }
}
