use ordered_float::OrderedFloat;

use crate::{
    error::{LoxError, LoxResult},
    model::token::{KEYWORDS, Token, TokenType},
};

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: u32,
    current: u32,
    line: u32,
}

impl Scanner {
    pub fn new(source: &String) -> Self {
        Self {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> LoxResult<&Vec<Token>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens
            .push(Token::new(TokenType::Eof, "".to_string(), self.line));
        Ok(&self.tokens)
    }

    fn is_at_end(&self) -> bool {
        return self.current >= self.source.len() as u32;
    }

    fn scan_token(&mut self) -> LoxResult<()> {
        let c = self.advance_char();
        match c {
            Some(c) => {
                match c {
                    '(' => self.add_token(TokenType::LeftParen),
                    ')' => self.add_token(TokenType::RightParen),
                    '{' => self.add_token(TokenType::LeftBrace),
                    '}' => self.add_token(TokenType::RightBrace),
                    ',' => self.add_token(TokenType::Comma),
                    '.' => self.add_token(TokenType::Dot),
                    '-' => self.add_token(TokenType::Minus),
                    '+' => self.add_token(TokenType::Plus),
                    ';' => self.add_token(TokenType::Semicolon),
                    '*' => self.add_token(TokenType::Star),
                    '!' => {
                        let typ = if self.match_char('=') {
                            TokenType::BangEqual
                        } else {
                            TokenType::Bang
                        };
                        self.add_token(typ);
                    }
                    '=' => {
                        let typ = if self.match_char('=') {
                            TokenType::EqualEqual
                        } else {
                            TokenType::Equal
                        };
                        self.add_token(typ);
                    }
                    '<' => {
                        let typ = if self.match_char('=') {
                            TokenType::LessEqual
                        } else {
                            TokenType::Less
                        };
                        self.add_token(typ);
                    }
                    '>' => {
                        let typ = if self.match_char('=') {
                            TokenType::GreaterEqual
                        } else {
                            TokenType::Greater
                        };
                        self.add_token(typ);
                    }
                    '/' => {
                        if self.match_char('/') {
                            while let Some(c) = self.peek_char()
                                && *c != '\n'
                            {
                                self.advance_char();
                            }
                        } else {
                            self.add_token(TokenType::Slash);
                        }
                    }
                    '"' => {
                        while let Some(c) = self.peek_char()
                            && *c != '"'
                        {
                            if *c == '\n' {
                                self.line += 1
                            }
                            self.advance_char();
                        }

                        if self.is_at_end() {
                            return Err(LoxError::ScanError {
                                message: format!("unterminated string, line: {}", self.line),
                            }
                            .into());
                        }

                        self.advance_char(); // must be '"'

                        let text = self.source
                            [(self.start + 1) as usize..(self.current - 1) as usize]
                            .iter()
                            .collect();

                        self.add_token(TokenType::String(text));
                    }
                    n if n.is_ascii_digit() => {
                        while let Some(c) = self.peek_char()
                            && (*c).is_ascii_digit()
                        {
                            self.advance_char();
                        }

                        if let Some(c) = self.peek_char()
                            && (*c) == '.'
                            && let Some(next_c) = self.peek_next_char()
                            && (*next_c).is_ascii_digit()
                        {}

                        while let Some(c) = self.peek_char()
                            && (*c).is_ascii_digit()
                        {
                            self.advance_char();
                        }

                        self.add_token(TokenType::Number(
                            self.source[self.start as usize..self.current as usize]
                                .iter()
                                .collect::<String>()
                                .parse::<f64>()
                                .map(|n| OrderedFloat(n))
                                .map_err(|_| LoxError::ScanError {
                                    message: format!("invalid number format, line: {}", self.line),
                                })?,
                        ));
                    }
                    n if n.is_ascii_alphabetic() || *n == '_' => {
                        while let Some(c) = self.peek_char()
                            && (c.is_ascii_alphanumeric() || *c == '_')
                        {
                            self.advance_char();
                        }
                        let text = self.source[self.start as usize..self.current as usize]
                            .iter()
                            .collect::<String>();
                        let typ = KEYWORDS.get(&text);
                        if let Some(typ) = typ {
                            self.add_token(typ.clone());
                        } else {
                            self.add_token(TokenType::Identifier(text));
                        }
                    }
                    ' ' => {}
                    '\r' => {}
                    '\t' => {}
                    '\n' => self.line += 1,
                    _ => {
                        return Err(LoxError::ScanError {
                            message: format!("unexpected character, line: {}", self.line),
                        }
                        .into());
                    }
                };
                Ok(())
            }
            None => Err(LoxError::ScanError {
                message: "not found char".into(),
            }
            .into()),
        }
    }

    fn advance_char(&mut self) -> Option<&char> {
        let res = self.source.get(self.current as usize);
        self.current += 1;
        res
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if let Some(c) = self.source.get(self.current as usize)
            && *c != expected
        {
            return false;
        }

        self.current += 1;
        return true;
    }

    fn peek_char(&self) -> Option<&char> {
        let res = self.source.get(self.current as usize);
        res
    }

    fn peek_next_char(&self) -> Option<&char> {
        let res = self.source.get((self.current + 1) as usize);
        res
    }

    fn add_token(&mut self, typ: TokenType) {
        let text = self.source[self.start as usize..self.current as usize]
            .iter()
            .collect();
        self.tokens.push(Token::new(typ, text, self.line));
    }
}
