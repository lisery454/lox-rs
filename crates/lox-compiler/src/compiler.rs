use crate::{
    error::{LoxError, LoxResult},
    model::{
        chunk::Chunk,
        local::Local,
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

    locals: Vec<Local>,
    scope_depth: i32,

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
            locals: Vec::new(),
            scope_depth: 0,
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
        let global_var_index = self.parse_var_to_index("Expect variable name.")?;

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

    fn parse_var_to_index(&mut self, msg: &str) -> LoxResult<u8> {
        self.consume(TokenType::Identifier, msg)?;
        // 局部变量直接返回，不需要向chunk添加数据，因为局部变量自动留在了stack中
        if self.scope_depth > 0 {
            self.declare_local_var()?;
            return Ok(0);
        }
        let p = self.get_previous_token();
        let s = p.lexeme;
        let index = self.add_constant(Constant::String(s));
        Ok(index)
    }

    fn define_var(&mut self, index: u8) -> LoxResult<()> {
        // 局部变量直接返回，不需要定义
        if self.scope_depth > 0 {
            // 这里算定义完成，把深度赋给它
            self.locals.last_mut().unwrap().depth = self.scope_depth;
            return Ok(());
        }
        self.emit_bytes(OpCode::DefineGlobal, index)
    }

    fn declare_local_var(&mut self) -> LoxResult<()> {
        let name = self.get_previous_token();

        for local in self.locals.iter().rev() {
            // 如果还没初始化或者深度小于当前的深度了，就说明已离开当前作用域了
            if local.depth != -1 && local.depth < self.scope_depth {
                break;
            }
            if name.lexeme == local.token.lexeme {
                return Err(LoxError::CompileError {
                    lexeme: name.lexeme,
                    line: name.line,
                    message: "Already a variable with this name in this scope.".into(),
                });
            }
        }

        self.add_local(name)?;
        Ok(())
    }

    fn add_local(&mut self, name: Token) -> LoxResult<()> {
        if self.locals.len() > 255 {
            return Err(LoxError::CompileError {
                lexeme: name.lexeme,
                line: name.line,
                message: "Too many local variables in function.".into(),
            });
        }
        // -1 depth 表示还没有初始化，只是声明
        self.locals.push(Local::new(name, -1));
        Ok(())
    }

    fn stmt(&mut self) -> LoxResult<()> {
        if self.match_(TokenType::Print)? {
            self.print_stmt()?
        } else if self.match_(TokenType::If)? {
            self.if_stmt()?
        } else if self.match_(TokenType::While)? {
            self.while_stmt()?
        } else if self.match_(TokenType::For)? {
            self.for_stmt()?
        } else if self.match_(TokenType::LeftBrace)? {
            self.begin_scope();
            self.block()?;
            self.end_scope()?;
        } else {
            self.expression_stmt()?
        }

        Ok(())
    }

    fn for_stmt(&mut self) -> LoxResult<()> {
        self.begin_scope();
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;

        if self.match_(TokenType::Semicolon)? {
            // no initializer
        } else if self.match_(TokenType::Var)? {
            self.var_decl()?;
        } else {
            self.expression_stmt()?;
        }

        let mut loop_start = self.current_chunk.count();
        let mut exit_jump_loc = None;
        if !self.match_(TokenType::Semicolon)? {
            self.expression()?;
            self.consume(TokenType::Semicolon, "Expect ';' after loop condition.")?;

            exit_jump_loc = Some(self.emit_jump(OpCode::JumpIfFalse)?);
            self.emit_byte(OpCode::Pop)?;
        }

        if !self.match_(TokenType::RightParen)? {
            let increment_jump_loc = self.emit_jump(OpCode::Jump)?;
            let increment_start = self.current_chunk.count();
            self.expression()?;
            self.emit_byte(OpCode::Pop)?;
            self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

            self.emit_loop(loop_start)?;
            loop_start = increment_start;
            self.patch_jump(increment_jump_loc)?;
        }

        self.stmt()?;
        self.emit_loop(loop_start)?;

        if let Some(exit_jump_loc) = exit_jump_loc {
            self.patch_jump(exit_jump_loc)?;
            self.emit_byte(OpCode::Pop)?;
        }
        self.end_scope()?;
        Ok(())
    }

    fn while_stmt(&mut self) -> LoxResult<()> {
        let loop_start = self.current_chunk.count();
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;

        let exit_jump_offset = self.emit_jump(OpCode::JumpIfFalse)?;
        self.emit_byte(OpCode::Pop)?;
        self.stmt()?;
        self.emit_loop(loop_start)?;
        self.patch_jump(exit_jump_offset)?;
        self.emit_byte(OpCode::Pop)?;
        Ok(())
    }

    fn if_stmt(&mut self) -> LoxResult<()> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;

        let then_jump_offset = self.emit_jump(OpCode::JumpIfFalse)?;
        {
            self.emit_byte(OpCode::Pop)?; // 清除条件值
            self.stmt()?;
        }
        let else_jump_offset = self.emit_jump(OpCode::Jump)?;
        self.patch_jump(then_jump_offset)?;
        {
            self.emit_byte(OpCode::Pop)?; // 清除条件值
            if self.match_(TokenType::Else)? {
                self.stmt()?;
            }
        }
        self.patch_jump(else_jump_offset)?;
        Ok(())
    }

    fn block(&mut self) -> LoxResult<()> {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration()?;
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(())
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

        let can_assign = prec <= Precedence::Assignment; // 当前是否是赋值优先级作用域，如果是比如加号表达式，说明优先级比赋值高，说明不能赋值
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

        // 如果是赋值优先级作用域，并且跟着等号，但是却没有被其他人消耗掉，说明有问题
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

// scope
impl Compiler {
    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) -> LoxResult<()> {
        self.scope_depth -= 1;

        // 需要把局部变量从stack中弹出，因为局部变量是存在stack中的
        while let Some(last_local) = self.locals.last()
            && last_local.depth > self.scope_depth
        {
            self.emit_byte(OpCode::Pop)?;
            self.locals.pop();
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

    fn emit_loop(&mut self, loop_start: usize) -> LoxResult<()> {
        self.emit_byte(OpCode::RevJump)?;
        let offset = self.current_chunk.count() + 2 - loop_start;
        if offset > u16::MAX as usize {
            return Err(crate::error::LoxError::CompileError {
                line: self.get_previous_token().line,
                lexeme: self.get_current_token().lexeme,
                message: "Loop body too large.".to_string(),
            });
        }

        self.emit_byte((offset >> 8) as u8 & 0xff)?;
        self.emit_byte(offset as u8 & 0xff)?;
        Ok(())
    }

    fn emit_jump<T: Into<u8>>(&mut self, instruction: T) -> LoxResult<usize> {
        self.emit_byte(instruction)?;
        self.emit_byte(0xff)?;
        self.emit_byte(0xff)?;
        // 返回的是记录jumpoffset指令的OpCode的offset
        Ok(self.current_chunk.count() - 2)
    }

    fn patch_jump(&mut self, loc: usize) -> LoxResult<()> {
        // 在读取需要jump的loc的OpCode后，需要jump的offset
        let jump = self.current_chunk.count() - loc - 2;

        if jump > u16::MAX as usize {
            return Err(crate::error::LoxError::CompileError {
                line: self.get_previous_token().line,
                lexeme: self.get_current_token().lexeme,
                message: "Too much code to jump over.".to_string(),
            });
        }

        self.current_chunk
            .overwrite(loc, ((jump >> 8) & 0xff) as u8);
        self.current_chunk
            .overwrite(loc + 1, (jump & 0xff) as u8);
        Ok(())
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
            ParseFnType::And => self.and(),
            ParseFnType::Or => self.or(),
        }
    }

    fn and(&mut self) -> LoxResult<()> {
        // 相当于if从句
        let jump_offset = self.emit_jump(OpCode::JumpIfFalse)?;
        // 如果左侧操作数为true，弹出stack顶部的操作数，继续运算
        self.emit_byte(OpCode::Pop)?;
        self.parse_precedence(Precedence::And)?;

        // 否则直接跳跃到最后
        self.patch_jump(jump_offset)?;
        Ok(())
    }

    fn or(&mut self) -> LoxResult<()> {
        // 相当于else从句
        let else_jump_offset = self.emit_jump(OpCode::JumpIfFalse)?;
        let end_jump_offset = self.emit_jump(OpCode::Jump)?;

        self.patch_jump(else_jump_offset)?;
        // 如果左侧操作数为false，弹出stack顶部的操作数，继续运算
        self.emit_byte(OpCode::Pop)?;
        self.parse_precedence(Precedence::Or)?;

        // 否则直接跳跃到最后
        self.patch_jump(end_jump_offset)?;

        Ok(())
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
        let get_op: OpCode;
        let set_op: OpCode;
        let index;

        // 如果是局部变量
        if let Some(i) = self.resolve_local(&name)? {
            index = i;
            get_op = OpCode::GetLocal;
            set_op = OpCode::SetLocal;
        }
        // 如果是全局变量
        else {
            index = self.add_constant(Constant::String(name.lexeme));
            get_op = OpCode::GetGlobal;
            set_op = OpCode::SetGlobal;
        }

        // 如果当前仍旧是赋值优先级作用域，就可以赋值；不然就是获取值。
        if can_assign && self.match_(TokenType::Equal)? {
            self.expression()?;
            self.emit_bytes(set_op, index)?;
        } else {
            self.emit_bytes(get_op, index)?;
        }
        Ok(())
    }

    fn resolve_local(&self, name: &Token) -> LoxResult<Option<u8>> {
        for (i, local) in self.locals.iter().enumerate() {
            if local.token.lexeme == name.lexeme {
                if local.depth == -1 {
                    return Err(LoxError::CompileError {
                        lexeme: name.lexeme.clone(),
                        line: name.line,
                        message: "Can't read local variable in its own initializer.".into(),
                    });
                }
                return Ok(Some(i as u8));
            }
        }

        return Ok(None);
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
