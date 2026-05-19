use crate::{
    error::{LoxError, LoxResult},
    model::token::{KEYWORDS, Token, TokenType},
};

pub struct Scanner {
    source: Vec<char>,
    // tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: &String) -> Self {
        Self {
            source: source.chars().collect(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan(&mut self) -> LoxResult<()> {
        let mut line = 0;
        loop {
            let token = self.scan_token()?;
            if token.line != line {
                print!("{:>4} ", token.line);
                line = token.line;
            } else {
                print!("   | ")
            }
            println!("{} '{}'", token.typ, token.lexeme);

            if token.typ == TokenType::Eof {
                break;
            }
        }
        Ok(())
    }

    pub fn scan_token(&mut self) -> LoxResult<Token> {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return Ok(self.make_token(TokenType::Eof));
        }

        let c = self.advance();

        let token = match c {
            '(' => self.make_token(TokenType::LeftParen),
            ')' => self.make_token(TokenType::RightParen),
            '{' => self.make_token(TokenType::LeftBrace),
            '}' => self.make_token(TokenType::RightBrace),
            ';' => self.make_token(TokenType::Semicolon),
            ',' => self.make_token(TokenType::Comma),
            '.' => self.make_token(TokenType::Dot),
            '-' => self.make_token(TokenType::Minus),
            '+' => self.make_token(TokenType::Plus),
            '*' => self.make_token(TokenType::Star),
            '/' => self.make_token(TokenType::Slash),
            '!' => {
                let typ = if self.match_advance('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.make_token(typ)
            }
            '=' => {
                let typ = if self.match_advance('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.make_token(typ)
            }
            '<' => {
                let typ = if self.match_advance('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.make_token(typ)
            }
            '>' => {
                let typ = if self.match_advance('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.make_token(typ)
            }
            '"' => {
                while let Some(p) = self.peek()
                    && *p != '"'
                {
                    if *p == '\n' {
                        self.line += 1;
                    }
                    self.advance();
                }

                if self.is_at_end() {
                    return Err(self.error("Unterminated string"));
                }

                self.advance();
                self.make_token(TokenType::String)
            }
            n if n.is_ascii_digit() => {
                while let Some(p) = self.peek()
                    && p.is_ascii_digit()
                {
                    self.advance();
                }

                if let Some(p) = self.peek()
                    && *p == '.'
                    && let Some(next) = self.peek_next()
                    && next.is_ascii_digit()
                {
                    self.advance();

                    while let Some(p) = self.peek()
                        && p.is_ascii_digit()
                    {
                        self.advance();
                    }
                }

                self.make_token(TokenType::Number)
            }
            n if n.is_ascii_alphabetic() || n == '_' => {
                while let Some(p) = self.peek()
                    && p.is_ascii_alphanumeric()
                {
                    self.advance();
                }

                let text = self.source[self.start..self.current]
                    .iter()
                    .collect::<String>();
                let typ = KEYWORDS.get(&text);
                if let Some(typ) = typ {
                    self.make_token(*typ)
                } else {
                    self.make_token(TokenType::Identifier)
                }
            }
            _ => {
                return Err(self.error("Unexpected character"));
            }
        };
        return Ok(token);
    }

    fn skip_whitespace(&mut self) {
        loop {
            if let Some(c) = self.peek() {
                match c {
                    ' ' | '\r' | '\t' => {
                        self.advance();
                    }
                    '\n' => {
                        self.advance();
                        self.line += 1;
                    }
                    '/' => {
                        if let Some('/') = self.peek_next() {
                            while let Some('\n') = self.peek() {
                                self.advance();
                            }
                        } else {
                            return;
                        }
                    }
                    _ => {
                        return;
                    }
                }
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<&char> {
        self.source.get(self.current)
    }

    fn peek_next(&self) -> Option<&char> {
        self.source.get(self.current + 1)
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        return self.source[self.current - 1];
    }

    fn match_advance(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source[self.current] != expected {
            return false;
        }

        self.current += 1;
        return true;
    }

    fn is_at_end(&self) -> bool {
        return self.current >= self.source.len();
    }

    fn make_token(&self, typ: TokenType) -> Token {
        Token {
            typ,
            lexeme: self.source[self.start..self.current].iter().collect(),
            line: self.line,
        }
    }

    fn error(&self, msg: &str) -> LoxError {
        crate::error::LoxError::ScanError {
            message: msg.to_string(),
            lexeme: self.source[self.start..self.current].iter().collect(),
            line: self.line,
        }
    }
}
