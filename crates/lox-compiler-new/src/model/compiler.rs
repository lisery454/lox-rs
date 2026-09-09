use anyhow::bail;

use crate::model::{
    Chunk, Constant, OpCode, ParseFnType, Precedence, Scanner, Token, TokenType, get_parse_rule,
};

pub struct Compiler {
    current: Option<Token>,
    previous: Option<Token>,

    scanner: Scanner,
    errors: Vec<anyhow::Error>,
}

impl Compiler {
    pub fn new(source: &str) -> Self {
        Self {
            scanner: Scanner::new(source),
            current: None,
            previous: None,
            errors: Vec::new(),
        }
    }

    pub fn compile(&mut self) -> anyhow::Result<Chunk> {
        let mut chunk = Chunk::new();
        self.advance();

        while !self.match_(TokenType::Eof) {
            if let Err(e) = self.declaration(&mut chunk) {
                self.errors.push(e);
                self.synchronize();
            }
        }

        self.emit_return(&mut chunk);

        if self.errors.len() > 0 {
            self.errors.iter().for_each(|e| eprintln!("{}", e));
            bail!("--- Compile Failed. ---")
        }

        Ok(chunk)
    }

    fn declaration(&mut self, chunk: &mut Chunk) -> anyhow::Result<()> {
        if self.match_(TokenType::Var) {
            self.var_decl()?;
        } else {
            self.stmt(chunk)?;
        }

        Ok(())
    }

    fn synchronize(&mut self) {
        while self.get_current_token().typ != TokenType::Eof {
            if self.get_previous_token().typ == TokenType::Semicolon {
                return;
            }

            match self.get_current_token().typ {
                TokenType::Class
                | TokenType::Function
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }

    fn var_decl(&mut self) -> anyhow::Result<()> {
        todo!()
        // let global_var_index = self.parse_var_to_index("Expect variable name.")?;

        // if self.match_(TokenType::Equal)? {
        //     self.expression()?;
        // } else {
        //     self.emit_byte(OpCode::Nil)?;
        // }

        // self.consume(
        //     TokenType::Semicolon,
        //     "Expect ';' after variable declaration.",
        // )?;

        // self.define_var(global_var_index)?;

        // Ok(())
    }

    fn stmt(&mut self, chunk: &mut Chunk) -> anyhow::Result<()> {
        if self.match_(TokenType::Print) {
            self.print_stmt(chunk)?
        // } else if self.match_(TokenType::If) {
        //     self.if_stmt()?
        // } else if self.match_(TokenType::While) {
        //     self.while_stmt()?
        // } else if self.match_(TokenType::For) {
        //     self.for_stmt()?
        // } else if self.match_(TokenType::LeftBrace) {
        //     self.begin_scope();
        //     self.block()?;
        //     self.end_scope()?;
        } else {
            self.expression_stmt(chunk)?;
        }

        Ok(())
    }

    fn print_stmt(&mut self, chunk: &mut Chunk) -> anyhow::Result<()> {
        self.expression(chunk)?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        self.emit_byte(chunk, OpCode::Print);
        Ok(())
    }

    fn expression_stmt(&mut self, chunk: &mut Chunk) -> anyhow::Result<()> {
        self.expression(chunk)?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        self.emit_byte(chunk, OpCode::Pop);
        Ok(())
    }

    fn expression(&mut self, chunk: &mut Chunk) -> anyhow::Result<()> {
        self.parse_precedence(chunk, Precedence::Assignment)?;
        Ok(())
    }
}

// utils
impl Compiler {
    fn get_previous_token(&self) -> &Token {
        self.previous.as_ref().unwrap()
    }

    fn get_current_token(&self) -> &Token {
        self.current.as_ref().unwrap()
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();

        loop {
            match self.scanner.scan_token() {
                Ok(t) => {
                    self.current = Some(t);
                    break;
                }
                Err(e) => {
                    self.errors.push(e);
                }
            }
        }
    }

    fn consume(&mut self, typ: TokenType, message: &str) -> anyhow::Result<()> {
        let t = self.get_current_token().clone();
        if t.typ == typ {
            self.advance();
            return Ok(());
        }

        bail!("{}, on word {}, in line {}", message, t.lexeme, t.line)
    }

    fn match_(&mut self, typ: TokenType) -> bool {
        if !self.check(typ) {
            return false;
        }
        self.advance();
        return true;
    }

    fn check(&self, typ: TokenType) -> bool {
        return self.get_current_token().typ == typ;
    }
}

// emit
impl Compiler {
    fn emit_byte<T: Into<u8>>(&self, chunk: &mut Chunk, byte: T) {
        chunk.write(byte, self.get_previous_token().line);
    }

    fn emit_return(&self, chunk: &mut Chunk) {
        self.emit_byte(chunk, OpCode::Return)
    }

    fn emit_bytes<T: Into<u8>, U: Into<u8>>(&self, chunk: &mut Chunk, byte1: T, byte2: U) {
        self.emit_byte(chunk, byte1);
        self.emit_byte(chunk, byte2);
    }

    fn emit_constant(&self, chunk: &mut Chunk, constant: Constant) {
        let index = chunk.add_constant(constant);
        self.emit_bytes(chunk, OpCode::Constant, index);
    }
}

// parse fn
impl Compiler {
    fn run_parse_fn(
        &mut self,
        chunk: &mut Chunk,
        typ: ParseFnType,
        can_assign: bool,
    ) -> anyhow::Result<()> {
        match typ {
            ParseFnType::Grouping => self.grouping(chunk),
            ParseFnType::Unary => self.unary(chunk),
            ParseFnType::Binary => self.binary(chunk),
            ParseFnType::Number => self.number(chunk),
            _ => todo!(), // ParseFnType::Literal => self.literal(),
                          // ParseFnType::String => self.string(),
                          // ParseFnType::Variable => self.variable(can_assign),
                          // ParseFnType::And => self.and(),
                          // ParseFnType::Or => self.or(),
        }
    }

    fn parse_precedence(&mut self, chunk: &mut Chunk, prec: Precedence) -> anyhow::Result<()> {
        self.advance();

        let prefix_fn_type = get_parse_rule(self.get_previous_token().typ).prefix;
        let Some(pft) = prefix_fn_type else {
            bail!(
                "Expect expression, on word {}, in line {}",
                self.get_previous_token().lexeme,
                self.get_previous_token().line
            );
        };

        let can_assign = prec <= Precedence::Assignment; // 当前是否是赋值优先级作用域，如果是比如加号表达式，说明优先级比赋值高，说明不能赋值
        self.run_parse_fn(chunk, pft, can_assign)?;

        while prec <= get_parse_rule(self.get_current_token().typ).precedence {
            self.advance();

            let Some(ift) = get_parse_rule(self.get_previous_token().typ).infix else {
                bail!(
                    "Expect expression, on word {}, in line {}",
                    self.get_previous_token().lexeme,
                    self.get_previous_token().line
                );
            };

            self.run_parse_fn(chunk, ift, can_assign)?;
        }

        // 如果是赋值优先级作用域，并且跟着等号，但是却没有被其他人消耗掉，说明有问题
        if can_assign && self.match_(TokenType::Equal) {
            bail!(
                "Invalid assignment target, on word {}, in line {}",
                self.get_previous_token().lexeme,
                self.get_previous_token().line
            );
        }

        Ok(())
    }

    fn number(&self, chunk: &mut Chunk) -> anyhow::Result<()> {
        let v = self.get_previous_token().lexeme.parse::<f64>()?;
        self.emit_constant(chunk, Constant::Number(v));
        Ok(())
    }

    fn grouping(&mut self, chunk: &mut Chunk) -> anyhow::Result<()> {
        self.expression(chunk)?;
        self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
        Ok(())
    }
    fn unary(&mut self, chunk: &mut Chunk) -> anyhow::Result<()> {
        let t = self.get_previous_token().clone();
        let op_typ = t.typ;
        self.expression(chunk)?;
        match op_typ {
            TokenType::Minus => self.emit_byte(chunk, OpCode::Negate),
            TokenType::Bang => self.emit_byte(chunk, OpCode::Not),
            _ => {
                bail!(
                    "invalid unary token type: {}, on word {}, in line {}",
                    op_typ,
                    t.lexeme,
                    t.line
                )
            }
        };
        Ok(())
    }

    fn binary(&mut self, chunk: &mut Chunk) -> anyhow::Result<()> {
        let t = self.get_previous_token().clone();
        let op_typ = t.typ;
        let rule = get_parse_rule(op_typ);
        self.parse_precedence(chunk, rule.precedence.higher())?;
        match op_typ {
            TokenType::Plus => self.emit_byte(chunk, OpCode::Add),
            TokenType::Minus => self.emit_byte(chunk, OpCode::Subtract),
            TokenType::Star => self.emit_byte(chunk, OpCode::Multiply),
            TokenType::Slash => self.emit_byte(chunk, OpCode::Divide),
            TokenType::BangEqual => self.emit_bytes(chunk, OpCode::Equal, OpCode::Not),
            TokenType::EqualEqual => self.emit_byte(chunk, OpCode::Equal),
            TokenType::Greater => self.emit_byte(chunk, OpCode::Greater),
            TokenType::GreaterEqual => self.emit_bytes(chunk, OpCode::Less, OpCode::Not),
            TokenType::Less => self.emit_byte(chunk, OpCode::Less),
            TokenType::LessEqual => self.emit_bytes(chunk, OpCode::Greater, OpCode::Not),
            _ => {
                bail!(
                    "invalid binary token type: {}, on word {}, in line {}",
                    op_typ,
                    t.lexeme,
                    t.line
                )
            }
        };
        Ok(())
    }
}
