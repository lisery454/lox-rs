use anyhow::bail;

use crate::{
    Scanner,
    model::{
        Chunk, Constant, Local, OpCode, ParseFnType, Precedence, Token, TokenType, get_parse_rule,
    },
};

pub struct Compiler {
    current: Option<Token>,
    previous: Option<Token>,

    locals: Vec<Local>,
    scope_depth: i32,

    scanner: Scanner,
    errors: Vec<anyhow::Error>,
}

impl Compiler {
    pub fn new(source: &str) -> Self {
        Self {
            scanner: Scanner::new(source),
            current: None,
            previous: None,
            locals: Vec::new(),
            errors: Vec::new(),
            scope_depth: 0,
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
            self.var_decl(chunk)?;
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

    fn var_decl(&mut self, chunk: &mut Chunk) -> anyhow::Result<()> {
        let global_var_index = self.parse_var_to_index(chunk, "Expect variable name.")?;

        if self.match_(TokenType::Equal) {
            self.expression(chunk)?;
        } else {
            self.emit_byte(chunk, OpCode::Nil);
        }

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;

        self.define_var(chunk, global_var_index)?;

        Ok(())
    }

    fn parse_var_to_index(&mut self, chunk: &mut Chunk, msg: &str) -> anyhow::Result<u8> {
        self.consume(TokenType::Identifier, msg)?;
        // 局部变量直接返回，不需要向chunk添加数据，因为局部变量自动留在了stack中
        if self.scope_depth > 0 {
            self.declare_local_var()?;
            return Ok(0);
        }
        let p = self.get_previous_token();
        let s = p.lexeme.clone();
        let index = self.add_constant(chunk, Constant::String(s));
        Ok(index)
    }

    fn declare_local_var(&mut self) -> anyhow::Result<()> {
        let name = self.get_previous_token().clone();

        for local in self.locals.iter().rev() {
            // 如果还没初始化或者深度小于当前的深度了，就说明已离开当前作用域了
            if local.depth != -1 && local.depth < self.scope_depth {
                break;
            }
            if name.lexeme == local.token.lexeme {
                bail!(
                    "Already a variable with this name in this scope, on word {}, in line {}.",
                    name.lexeme,
                    name.line
                );
            }
        }

        self.add_local(name)?;
        Ok(())
    }

    fn add_local(&mut self, name: Token) -> anyhow::Result<()> {
        if self.locals.len() > 255 {
            bail!(
                "Too many local variables in function, on word {}, in line {}.",
                name.lexeme,
                name.line
            );
        }
        // -1 depth 表示还没有初始化，只是声明
        self.locals.push(Local::new(name, -1));
        Ok(())
    }

    fn define_var(&mut self, chunk: &mut Chunk, index: u8) -> anyhow::Result<()> {
        // // 局部变量直接返回，不需要定义
        if self.scope_depth > 0 {
            // 这里算定义完成，把深度赋给它
            self.locals.last_mut().unwrap().depth = self.scope_depth;
            return Ok(());
        }
        self.emit_bytes(chunk, OpCode::DefineGlobal, index);
        Ok(())
    }

    fn stmt(&mut self, chunk: &mut Chunk) -> anyhow::Result<()> {
        if self.match_(TokenType::Print) {
            self.print_stmt(chunk)?
        } else if self.match_(TokenType::If) {
            self.if_stmt(chunk)?
        // } else if self.match_(TokenType::While) {
        //     self.while_stmt()?
        // } else if self.match_(TokenType::For) {
        //     self.for_stmt()?
        } else if self.match_(TokenType::LeftBrace) {
            self.begin_scope();
            self.block(chunk)?;
            self.end_scope(chunk)?;
        } else {
            self.expression_stmt(chunk)?;
        }

        Ok(())
    }

    fn if_stmt(&mut self, chunk: &mut Chunk) -> anyhow::Result<()> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        self.expression(chunk)?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;

        let then_jump_index = self.emit_jump(chunk, OpCode::JumpIfFalse)?;
        {
            self.emit_byte(chunk, OpCode::Pop); // 清除条件值
            self.stmt(chunk)?;
        }
        let else_jump_index = self.emit_jump(chunk, OpCode::Jump)?;
        self.patch_jump(chunk, then_jump_index)?;
        {
            self.emit_byte(chunk, OpCode::Pop); // 清除条件值
            if self.match_(TokenType::Else) {
                self.stmt(chunk)?;
            }
        }
        self.patch_jump(chunk, else_jump_index)?;
        Ok(())
    }

    fn block(&mut self, chunk: &mut Chunk) -> anyhow::Result<()> {
        while !self.check(TokenType::RightBrace) && !self.check(TokenType::Eof) {
            self.declaration(chunk)?;
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self, chunk: &mut Chunk) -> anyhow::Result<()> {
        self.scope_depth -= 1;

        // 局部变量存在 stack 中，离开作用域时要把它们从 stack 弹出
        while let Some(last_local) = self.locals.last()
            && last_local.depth > self.scope_depth
        {
            self.emit_byte(chunk, OpCode::Pop);
            self.locals.pop();
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
        let b = byte.into();
        chunk.write(b, self.get_previous_token().line);
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

    fn add_constant(&mut self, chunk: &mut Chunk, constant: Constant) -> u8 {
        let index = chunk.add_constant(constant);
        index
    }

    fn emit_jump<T: Into<u8>>(
        &mut self,
        chunk: &mut Chunk,
        instruction: T,
    ) -> anyhow::Result<usize> {
        self.emit_byte(chunk, instruction);
        self.emit_byte(chunk, 0xff);
        self.emit_byte(chunk, 0xff);
        // 返回的是记录jumpoffset指令的OpCode的offset
        Ok(chunk.count() - 2)
    }

    fn patch_jump(&mut self, chunk: &mut Chunk, code_index: usize) -> anyhow::Result<()> {
        // 在读取需要jump的loc的OpCode后，需要jump的offset
        let jump = chunk.count() - code_index - 2;

        if jump > u16::MAX as usize {
            bail!(
                "Too much code to jump over, on word {}, in line {}",
                self.get_current_token().lexeme,
                self.get_previous_token().line
            );
        }

        chunk.overwrite(code_index, ((jump >> 8) & 0xff) as u8);
        chunk.overwrite(code_index + 1, (jump & 0xff) as u8);
        Ok(())
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
            ParseFnType::Literal => self.literal(chunk),
            ParseFnType::String => self.string(chunk),
            ParseFnType::Variable => self.variable(chunk, can_assign),
            // ParseFnType::And => self.and(chunk),
            // ParseFnType::Or => self.or(chunk),
            _ => todo!(),
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

    fn variable(&mut self, chunk: &mut Chunk, can_assign: bool) -> anyhow::Result<()> {
        let name = self.get_previous_token().clone();
        self.named_var(chunk, name, can_assign)
    }

    fn named_var(
        &mut self,
        chunk: &mut Chunk,
        name: Token,
        can_assign: bool,
    ) -> anyhow::Result<()> {
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
            index = self.add_constant(chunk, Constant::String(name.lexeme));
            get_op = OpCode::GetGlobal;
            set_op = OpCode::SetGlobal;
        }

        // 如果当前仍旧是赋值优先级作用域，就可以赋值；不然就是获取值。
        if can_assign && self.match_(TokenType::Equal) {
            self.expression(chunk)?;
            self.emit_bytes(chunk, set_op, index);
        } else {
            self.emit_bytes(chunk, get_op, index);
        }
        Ok(())
    }

    fn resolve_local(&self, name: &Token) -> anyhow::Result<Option<u8>> {
        // 从后往前找，最内层作用域的变量优先
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.token.lexeme == name.lexeme {
                if local.depth == -1 {
                    bail!(
                        "Can't read local variable in its own initializer, on {}, in {}",
                        name.lexeme,
                        name.line
                    );
                }
                return Ok(Some(i as u8));
            }
        }

        return Ok(None);
    }

    fn literal(&mut self, chunk: &mut Chunk) -> anyhow::Result<()> {
        let t = self.get_previous_token().clone();
        let typ = t.typ;
        match typ {
            TokenType::False => self.emit_byte(chunk, OpCode::False),
            TokenType::True => self.emit_byte(chunk, OpCode::True),
            TokenType::Nil => self.emit_byte(chunk, OpCode::Nil),
            _ => {
                bail!(
                    "invalid literal type: {}, on word {}, in line {}",
                    typ,
                    t.lexeme,
                    t.line
                )
            }
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

    fn string(&mut self, chunk: &mut Chunk) -> anyhow::Result<()> {
        let token = self.get_previous_token().clone();
        let s = token.lexeme.trim_matches('"').to_string();
        self.emit_constant(chunk, Constant::String(s));
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
