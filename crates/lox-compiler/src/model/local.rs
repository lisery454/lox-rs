use crate::model::token::Token;

pub struct Local {
    pub(crate) token: Token,
    pub(crate) depth: i32,
}

impl Local {
    pub fn new(token: Token, depth: i32) -> Self {
        Local { token, depth }
    }
}
