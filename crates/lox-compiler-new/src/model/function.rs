use crate::model::Chunk;

#[derive(Clone)]
pub struct Function {
    pub arity: usize, // pram size
    pub chunk: Chunk, // body
    pub name: String, // function name
}

impl Function {
    pub fn new() -> Self {
        Self {
            arity: 0,
            chunk: Chunk::new(),
            name: String::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum FuncType {
    Function,
    Script,
}
