use crate::model::{Constant, OpCode};

#[derive(Clone)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub lines: Vec<usize>,
    pub constants: Vec<Constant>,
}

impl Chunk {
    const MAX_CONSTANT_LENGTH: usize = 256;

    pub fn new() -> Self {
        return Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        };
    }

    pub fn write<B: Into<u8>>(&mut self, byte: B, line: usize) {
        self.code.push(byte.into());
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, v: Constant) -> u8 {
        let index = self.constants.len();
        self.constants.push(v);
        if index < Self::MAX_CONSTANT_LENGTH {
            return index as u8;
        }
        panic!("Too many constants in one chunk!");
    }

    pub fn count(&self) -> usize {
        return self.code.len();
    }

    pub fn overwrite<T: Into<u8>>(&mut self, loc: usize, t: T) {
        self.code[loc] = t.into();
    }

    pub fn with_ip(&self, ip: i32) -> ChunkWithIp<'_> {
        ChunkWithIp { ip, chunk: &self }
    }
}

impl std::fmt::Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.with_ip(-1))
    }
}

pub struct ChunkWithIp<'a> {
    pub ip: i32,
    pub chunk: &'a Chunk,
}

impl<'a> std::fmt::Display for ChunkWithIp<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut offset: usize = 0;
        while offset < self.chunk.code.len() {
            let code = self.chunk.code[offset];

            let is_highlight = offset as i32 == self.ip;
            if is_highlight {
                write!(f, " -> ")?;
            } else {
                write!(f, "    ")?;
            }

            let is_new_line =
                offset == 0 || self.chunk.lines[offset] != self.chunk.lines[offset - 1];
            if is_new_line {
                write!(f, "[ {:4} ] ", self.chunk.lines[offset])?;
            } else {
                write!(f, "[    | ] ")?;
            }

            match OpCode::try_from(code) {
                Ok(code) => match code {
                    OpCode::Constant
                    | OpCode::DefineGlobal
                    | OpCode::GetGlobal
                    | OpCode::SetGlobal => {
                        write!(f, "{:04} ", offset)?;

                        let value_index = self.chunk.code[offset + 1] as usize;
                        let constants = &self.chunk.constants;
                        let value = constants.get(value_index);
                        if let Some(v) = value {
                            write!(f, "{}({})", code, v)?;
                        } else {
                            panic!("invalid constant index");
                        }
                        offset += 2;
                    }
                    OpCode::GetLocal | OpCode::SetLocal => {
                        write!(f, "{:04} ", offset)?;

                        let slot = self.chunk.code[offset + 1];
                        write!(f, "{}(slot {})", code, slot)?;
                        offset += 2;
                    }
                    OpCode::Call => {
                        write!(f, "{:04} ", offset)?;

                        let arg_count = self.chunk.code[offset + 1];
                        write!(f, "{}({})", code, arg_count)?;
                        offset += 2;
                    }
                    OpCode::JumpIfFalse | OpCode::Jump => {
                        write!(f, "{:04} ", offset)?;

                        let byte = self.chunk.code[offset + 1] as usize;
                        let byte2 = self.chunk.code[offset + 2] as usize;
                        let jump_to = ((byte << 8) | byte2) + offset + 3;

                        write!(f, "{}({})", code, jump_to)?;
                        offset += 3;
                    }
                    OpCode::RevJump => {
                        write!(f, "{:04} ", offset)?;

                        let byte = self.chunk.code[offset + 1] as usize;
                        let byte2 = self.chunk.code[offset + 2] as usize;

                        let jump_to = offset + 3 - ((byte << 8) | byte2);

                        write!(f, "{}({})", code, jump_to)?;
                        offset += 3;
                    }
                    _ => {
                        write!(f, "{:04} ", offset)?;
                        write!(f, "{}", code)?;

                        offset += 1;
                    }
                },
                Err(_) => {
                    panic!("invalid op code: {}", code)
                }
            };

            if offset < self.chunk.code.len() {
                writeln!(f, "")?;
            }
        }

        Ok(())
    }
}
