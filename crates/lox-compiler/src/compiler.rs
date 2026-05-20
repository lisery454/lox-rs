use crate::{
    error::{LoxError, LoxResult},
    model::{
        chunk::Chunk,
        opcode::OpCode,
        parse_rule::{ParseFnType, get_parse_rule},
        precedence::Precedence,
        token::{Token, TokenType},
        value::Constant,
    },
    scanner::Scanner,
};

pub struct Compiler {
    current: Option<Token>,
    previous: Option<Token>,
    scanner: Scanner,
    current_chunk: Chunk,
    errors: Vec<LoxError>,
}

impl Compiler {
    pub fn new(source: &String) -> Self {
        Self {
            current: None,
            previous: None,
            scanner: Scanner::new(source),
            current_chunk: Chunk::new(),
            errors: Vec::new(),
        }
    }

    fn get_previous_token(&self) -> Token {
        self.previous.clone().unwrap()
    }

    fn get_current_token(&self) -> Token {
        self.current.clone().unwrap()
    }

    pub fn compile(&mut self) -> LoxResult<Chunk> {
        self.advance()?;

        while !self.match_(TokenType::Eof)? {
            self.declaration()?;
        }

        self.emit_return()?;

        if self.errors.len() > 0 {
            return Err(LoxError::MergeError {
                errors: std::mem::take(&mut self.errors),
            });
        }

        Ok(self.current_chunk.clone())
    }

    fn declaration(&mut self) -> LoxResult<()> {
        let res = if self.match_(TokenType::Var)? {
            self.var_decl()
        } else {
            self.stmt()
        };

        if let Err(e) = res {
            self.errors.push(e);
            self.synchronize()?;
        }

        Ok(())
    }

    fn var_decl(&mut self) -> LoxResult<()> {
        let global_var_index = self.parse_var("Expect variable name.")?;

        if self.match_(TokenType::Equal)? {
            self.expression()?;
        } else {
            self.emit_byte(OpCode::Nil)?;
        }

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;

        self.define_var(global_var_index)?;

        Ok(())
    }

    fn parse_var(&mut self, msg: &str) -> LoxResult<u8> {
        self.consume(TokenType::Identifier, msg)?;
        let p = self.get_previous_token();
        let s = p.lexeme;
        let index = self.add_constant(Constant::String(s));
        Ok(index)
    }

    fn define_var(&mut self, index: u8) -> LoxResult<()> {
        self.emit_bytes(OpCode::DefineGlobal, index)
    }

    fn stmt(&mut self) -> LoxResult<()> {
        if self.match_(TokenType::Print)? {
            self.print_stmt()
        } else {
            self.expression_stmt()
        }
    }

    fn synchronize(&mut self) -> LoxResult<()> {
        while self.get_current_token().typ != TokenType::Eof {
            if self.get_previous_token().typ == TokenType::Semicolon {
                return Ok(());
            }

            match self.get_current_token().typ {
                TokenType::Class
                | TokenType::Function
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return Ok(()),
                _ => {}
            }

            self.advance()?;
        }
        Ok(())
    }

    fn print_stmt(&mut self) -> LoxResult<()> {
        self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        self.emit_byte(OpCode::Print)?;
        Ok(())
    }

    fn expression_stmt(&mut self) -> LoxResult<()> {
        self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        self.emit_byte(OpCode::Pop)?;
        Ok(())
    }

    fn expression(&mut self) -> LoxResult<()> {
        self.parse_precedence(Precedence::Assignment)?;
        Ok(())
    }

    fn parse_precedence(&mut self, prec: Precedence) -> LoxResult<()> {
        self.advance()?;

        let prefix_fn_type = get_parse_rule(self.get_previous_token().typ).prefix;
        let Some(pft) = prefix_fn_type else {
            return Err(crate::error::LoxError::CompileError {
                line: self.get_previous_token().line,
                lexeme: self.get_current_token().lexeme,
                message: "Expect expression.".to_string(),
            });
        };

        let can_assign = prec <= Precedence::Assignment;
        self.run_parse_fn(pft, can_assign)?;

        while prec <= get_parse_rule(self.get_current_token().typ).precedence {
            self.advance()?;

            let Some(ift) = get_parse_rule(self.get_previous_token().typ).infix else {
                return Err(crate::error::LoxError::CompileError {
                    line: self.get_previous_token().line,
                    lexeme: self.get_current_token().lexeme,
                    message: "Expect expression.".to_string(),
                });
            };

            self.run_parse_fn(ift, can_assign)?;
        }

        if can_assign && self.match_(TokenType::Equal)? {
            return Err(crate::error::LoxError::CompileError {
                line: self.get_previous_token().line,
                lexeme: self.get_current_token().lexeme,
                message: "Invalid assignment target.".to_string(),
            });
        }

        Ok(())
    }
}

// utils
impl Compiler {
    fn advance(&mut self) -> LoxResult<()> {
        self.previous = self.current.clone();
        let mut errors = vec![];
        loop {
            match self.scanner.scan_token() {
                Ok(t) => {
                    self.current = Some(t);
                    break;
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        if let Some(e) = errors.get(0) {
            eprintln!("{}", e);
        }

        Ok(())
    }

    fn consume(&mut self, typ: TokenType, message: &str) -> LoxResult<()> {
        let t = self.get_current_token();
        if t.typ == typ {
            self.advance()?;
            return Ok(());
        }

        Err(crate::error::LoxError::CompileError {
            line: t.line,
            lexeme: t.lexeme,
            message: message.to_string(),
        })
    }

    fn match_(&mut self, typ: TokenType) -> LoxResult<bool> {
        if !self.check(typ) {
            return Ok(false);
        }
        self.advance()?;
        return Ok(true);
    }

    fn check(&mut self, typ: TokenType) -> bool {
        return self.get_current_token().typ == typ;
    }
}

// emit
impl Compiler {
    fn emit_byte<T: Into<u8>>(&mut self, byte: T) -> LoxResult<()> {
        self.current_chunk
            .write(byte, self.get_previous_token().line);
        Ok(())
    }

    fn emit_return(&mut self) -> LoxResult<()> {
        self.emit_byte(OpCode::Return)
    }

    fn emit_bytes<T: Into<u8>, U: Into<u8>>(&mut self, byte1: T, byte2: U) -> LoxResult<()> {
        self.emit_byte(byte1)?;
        self.emit_byte(byte2)?;
        Ok(())
    }

    fn emit_constant(&mut self, constant: Constant) -> LoxResult<()> {
        let index = self.add_constant(constant);
        self.emit_bytes(OpCode::Constant, index)?;
        Ok(())
    }

    fn add_constant(&mut self, constant: Constant) -> u8 {
        let index = self.current_chunk.add_constant(constant);
        index
    }
}

// parse fn
impl Compiler {
    fn run_parse_fn(&mut self, typ: ParseFnType, can_assign: bool) -> LoxResult<()> {
        match typ {
            ParseFnType::Grouping => self.grouping(),
            ParseFnType::Unary => self.unary(),
            ParseFnType::Binary => self.binary(),
            ParseFnType::Number => self.number(),
            ParseFnType::Literal => self.literal(),
            ParseFnType::String => self.string(),
            ParseFnType::Variable => self.variable(can_assign),
        }
    }

    fn string(&mut self) -> LoxResult<()> {
        let token = self.get_previous_token();
        let s = token.lexeme.trim_matches('"').to_string();
        self.emit_constant(Constant::String(s))?;
        Ok(())
    }

    fn variable(&mut self, can_assign: bool) -> LoxResult<()> {
        let name = self.get_previous_token();
        self.named_var(name, can_assign)
    }

    fn named_var(&mut self, name: Token, can_assign: bool) -> LoxResult<()> {
        let i = self.add_constant(Constant::String(name.lexeme));

        if can_assign && self.match_(TokenType::Equal)? {
            self.expression()?;
            self.emit_bytes(OpCode::SetGlobal, i)?;
        } else {
            self.emit_bytes(OpCode::GetGlobal, i)?;
        }
        Ok(())
    }

    fn literal(&mut self) -> LoxResult<()> {
        let t = self.get_previous_token();
        let typ = t.typ;
        match typ {
            TokenType::False => self.emit_byte(OpCode::False),
            TokenType::True => self.emit_byte(OpCode::True),
            TokenType::Nil => self.emit_byte(OpCode::Nil),
            _ => Err(crate::error::LoxError::CompileError {
                line: t.line,
                lexeme: t.lexeme,
                message: "invalid literal type".to_string(),
            }),
        }
    }

    fn number(&mut self) -> LoxResult<()> {
        let v = self.get_previous_token().lexeme.parse::<f64>()?;
        self.emit_constant(Constant::Number(v))?;
        Ok(())
    }

    fn grouping(&mut self) -> LoxResult<()> {
        self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
        Ok(())
    }

    fn unary(&mut self) -> LoxResult<()> {
        let t = self.get_previous_token();
        let op_typ = t.typ;
        self.parse_precedence(Precedence::Unary)?;
        match op_typ {
            TokenType::Minus => self.emit_byte(OpCode::Negate),
            TokenType::Bang => self.emit_byte(OpCode::Not),
            _ => {
                return Err(crate::error::LoxError::CompileError {
                    line: t.line,
                    lexeme: t.lexeme,
                    message: format!("invalid unary token type: {}", op_typ),
                });
            }
        }?;
        Ok(())
    }

    fn binary(&mut self) -> LoxResult<()> {
        let t = self.get_previous_token();
        let op_typ = t.typ;
        let rule = get_parse_rule(op_typ);
        self.parse_precedence(rule.precedence.higher())?;
        match op_typ {
            TokenType::Plus => self.emit_byte(OpCode::Add),
            TokenType::Minus => self.emit_byte(OpCode::Subtract),
            TokenType::Star => self.emit_byte(OpCode::Multiply),
            TokenType::Slash => self.emit_byte(OpCode::Divide),
            TokenType::BangEqual => self.emit_bytes(OpCode::Equal, OpCode::Not),
            TokenType::EqualEqual => self.emit_byte(OpCode::Equal),
            TokenType::Greater => self.emit_byte(OpCode::Greater),
            TokenType::GreaterEqual => self.emit_bytes(OpCode::Less, OpCode::Not),
            TokenType::Less => self.emit_byte(OpCode::Less),
            TokenType::LessEqual => self.emit_bytes(OpCode::Greater, OpCode::Not),
            _ => {
                return Err(crate::error::LoxError::CompileError {
                    line: t.line,
                    lexeme: t.lexeme,
                    message: format!("invalid binary token type: {}", op_typ),
                });
            }
        }?;
        Ok(())
    }
}
