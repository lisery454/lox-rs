use std::rc::Rc;

use crate::{
    error::LoxResult,
    model::{
        chunk::Chunk,
        opcode::OpCode,
        parse_rule::{ParseFnType, get_parse_rule},
        precedence::Precedence,
        token::{Token, TokenType},
        value::{Constant, Obj, Value},
    },
    scanner::Scanner,
};

pub struct Compiler {
    current: Option<Token>,
    previous: Option<Token>,
    scanner: Scanner,
    current_chunk: Chunk,
}

impl Compiler {
    pub fn new(source: &String) -> Self {
        Self {
            current: None,
            previous: None,
            scanner: Scanner::new(source),
            current_chunk: Chunk::new(),
        }
    }

    pub fn compile(&mut self) -> LoxResult<Chunk> {
        self.advance()?;
        self.expression()?;
        self.consume(TokenType::Eof, "Expect end of expression")?;
        self.emit_return()?;

        Ok(self.current_chunk.clone())
    }

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

    fn get_previous_token(&self) -> Token {
        self.previous.clone().unwrap()
    }

    fn get_current_token(&self) -> Token {
        self.current.clone().unwrap()
    }

    fn consume(&mut self, typ: TokenType, message: &str) -> LoxResult<()> {
        if let Some(t) = &self.current
            && t.typ == typ
        {
            self.advance()?;
            return Ok(());
        }

        Err(crate::error::LoxError::CompileError(message.to_string()))
    }

    fn expression(&mut self) -> LoxResult<()> {
        self.parse_precedence(Precedence::Assignment)?;
        Ok(())
    }

    fn parse_precedence(&mut self, prec: Precedence) -> LoxResult<()> {
        self.advance()?;

        let prefix_fn_type = get_parse_rule(self.get_previous_token().typ).prefix;
        let Some(pft) = prefix_fn_type else {
            return Err(crate::error::LoxError::CompileError(
                "Expect expression.".to_string(),
            ));
        };
        self.run_parse_fn(pft)?;

        while prec <= get_parse_rule(self.get_current_token().typ).precedence {
            self.advance()?;

            let Some(ift) = get_parse_rule(self.get_previous_token().typ).infix else {
                return Err(crate::error::LoxError::CompileError(
                    "Expect expression 2.".to_string(),
                ));
            };

            self.run_parse_fn(ift)?;
        }

        Ok(())
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
        let index = self.current_chunk.add_constant(constant);
        self.emit_bytes(OpCode::Constant, index)?;
        Ok(())
    }
}

// parse fn
impl Compiler {
    fn run_parse_fn(&mut self, typ: ParseFnType) -> LoxResult<()> {
        match typ {
            ParseFnType::Grouping => self.grouping(),
            ParseFnType::Unary => self.unary(),
            ParseFnType::Binary => self.binary(),
            ParseFnType::Number => self.number(),
            ParseFnType::Literal => self.literal(),
            ParseFnType::String => self.string(),
        }
    }

    fn string(&mut self) -> LoxResult<()> {
        let token = self.get_previous_token();
        let s = token.lexeme.trim_matches('"').to_string();
        self.emit_constant(Constant::String(s));
        Ok(())
    }

    fn literal(&mut self) -> LoxResult<()> {
        let typ = self.get_previous_token().typ;
        match typ {
            TokenType::False => self.emit_byte(OpCode::False),
            TokenType::True => self.emit_byte(OpCode::True),
            TokenType::Nil => self.emit_byte(OpCode::Nil),
            _ => Err(crate::error::LoxError::CompileError(
                "invalid literal type".into(),
            )),
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
        let op_typ = self.get_previous_token().typ;
        self.parse_precedence(Precedence::Unary)?;
        match op_typ {
            TokenType::Minus => self.emit_byte(OpCode::Negate),
            TokenType::Bang => self.emit_byte(OpCode::Not),
            _ => {
                return Err(crate::error::LoxError::CompileError(format!(
                    "invalid unary token type: {}",
                    op_typ
                )));
            }
        }?;
        Ok(())
    }

    fn binary(&mut self) -> LoxResult<()> {
        let op_typ = self.get_previous_token().typ;
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
                return Err(crate::error::LoxError::CompileError(format!(
                    "invalid binary token type: {}",
                    op_typ
                )));
            }
        }?;
        Ok(())
    }
}
