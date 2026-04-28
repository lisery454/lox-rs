use std::mem::discriminant;

use anyhow::Result;

use crate::{
    error::LoxError,
    model::{
        expr::{
            BinaryExprData, Expr, GroupingExprData, LiteralExprData, LiteralValue, UnaryExprData,
        },
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

    pub fn parse(&mut self) -> Option<Expr> {
        self.parse_experssion().ok()
    }

    fn parse_experssion(&mut self) -> Result<Expr> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut expr: Expr = self.parse_comparison()?;

        while let Some(token) = self.peek()
            && matches!(token.typ, TokenType::BangEqual | TokenType::EqualEqual)
        {
            self.advance();
            let operator = (*self.previous().unwrap()).clone();
            let right = self.parse_comparison()?;
            expr = Expr::Binary(BinaryExprData {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut expr = self.parse_term()?;

        while let Some(token) = self.peek()
            && matches!(
                token.typ,
                TokenType::Greater
                    | TokenType::GreaterEqual
                    | TokenType::Less
                    | TokenType::LessEqual
            )
        {
            self.advance();
            let operator = (*self.previous().unwrap()).clone();
            let right = self.parse_term()?;
            expr = Expr::Binary(BinaryExprData {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr> {
        let mut expr = self.parse_factor()?;

        while let Some(token) = self.peek()
            && matches!(token.typ, TokenType::Plus | TokenType::Minus)
        {
            self.advance();
            let operator = (*self.previous().unwrap()).clone();
            let right = self.parse_factor()?;
            expr = Expr::Binary(BinaryExprData {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr> {
        let mut expr = self.parse_unary()?;

        while let Some(token) = self.peek()
            && matches!(token.typ, TokenType::Star | TokenType::Slash)
        {
            self.advance();
            let operator = (*self.previous().unwrap()).clone();
            let right = self.parse_unary()?;
            expr = Expr::Binary(BinaryExprData {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        if let Some(token) = self.peek()
            && matches!(token.typ, TokenType::Minus | TokenType::Bang)
        {
            self.advance();
            let operator = (*self.previous().unwrap()).clone();
            let right = self.parse_unary()?;
            return Ok(Expr::Unary(UnaryExprData {
                operator,
                right: Box::new(right),
            }));
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        if let Some(token) = self.peek() {
            if matches!(token.typ, TokenType::False) {
                self.advance();
                return Ok(Expr::Literal(LiteralExprData {
                    value: LiteralValue::Bool(false),
                }));
            }
            if matches!(token.typ, TokenType::True) {
                self.advance();
                return Ok(Expr::Literal(LiteralExprData {
                    value: LiteralValue::Bool(true),
                }));
            }
            if matches!(token.typ, TokenType::Nil) {
                self.advance();
                return Ok(Expr::Literal(LiteralExprData {
                    value: LiteralValue::Nil,
                }));
            }
            if let TokenType::Number(n) = token.typ.clone() {
                self.advance();
                return Ok(Expr::Literal(LiteralExprData {
                    value: LiteralValue::Number(n),
                }));
            }
            if let TokenType::String(n) = token.typ.clone() {
                self.advance();
                return Ok(Expr::Literal(LiteralExprData {
                    value: LiteralValue::String(n),
                }));
            }
            if matches!(token.typ, TokenType::LeftParen) {
                self.advance();
                let expr = self.parse_experssion()?;
                self.consume(TokenType::RightParen, "not find right paren <}>".into())?;
                return Ok(Expr::Grouping(GroupingExprData {
                    expression: Box::new(expr),
                }));
            }
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

    fn consume(&mut self, typ: TokenType, message: String) -> Result<Option<&Token>> {
        if let Some(t) = self.peek()
            && discriminant(&typ) == discriminant(&t.typ)
        {
            return Ok(self.advance());
        }

        return Err(LoxError::ParseError { message: message }.into());
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

    fn previous(&self) -> Option<&Token> {
        if self.current == 0 {
            return None;
        }
        self.tokens.get((self.current - 1) as usize)
    }
}
