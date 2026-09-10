use crate::model::token::Token;

// -1 depth 表示还没有初始化，只是声明
pub struct Local {
    pub(crate) token: Token,
    pub(crate) depth: i32,
}

impl Local {
    pub fn new(token: Token, depth: i32) -> Self {
        Local { token, depth }
    }
}
