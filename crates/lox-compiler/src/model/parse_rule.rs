use std::sync::LazyLock;

use strum::EnumCount;

use crate::model::{precedence::Precedence, token::TokenType};

#[derive(Clone, Copy)]
pub enum ParseFnType {
    Grouping,
    Unary,
    Binary,
    Number,
    Literal,
}

#[derive(Clone, Copy)]
pub struct ParseRule {
    pub(crate) prefix: Option<ParseFnType>,
    pub(crate) infix: Option<ParseFnType>,
    pub(crate) precedence: Precedence,
}

const PARSE_RULES: LazyLock<[ParseRule; TokenType::COUNT]> = LazyLock::new(|| {
    let none_rule = ParseRule {
        prefix: None,
        infix: None,
        precedence: Precedence::None,
    };
    std::array::from_fn(|index| match TokenType::try_from(index) {
        Ok(typ) => match typ {
            TokenType::LeftParen => ParseRule {
                prefix: Some(ParseFnType::Grouping),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::RightParen => none_rule,
            TokenType::LeftBrace => none_rule,
            TokenType::RightBrace => none_rule,
            TokenType::Comma => none_rule,
            TokenType::Dot => none_rule,
            TokenType::Minus => ParseRule {
                prefix: Some(ParseFnType::Unary),
                infix: Some(ParseFnType::Binary),
                precedence: Precedence::Term,
            },
            TokenType::Plus => ParseRule {
                prefix: None,
                infix: Some(ParseFnType::Binary),
                precedence: Precedence::Term,
            },
            TokenType::Semicolon => none_rule,
            TokenType::Slash => ParseRule {
                prefix: None,
                infix: Some(ParseFnType::Binary),
                precedence: Precedence::Factor,
            },
            TokenType::Star => ParseRule {
                prefix: None,
                infix: Some(ParseFnType::Binary),
                precedence: Precedence::Factor,
            },
            TokenType::Bang => ParseRule {
                prefix: Some(ParseFnType::Unary),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::BangEqual => ParseRule {
                prefix: None,
                infix: Some(ParseFnType::Binary),
                precedence: Precedence::Equality,
            },
            TokenType::Equal => none_rule,
            TokenType::EqualEqual => ParseRule {
                prefix: None,
                infix: Some(ParseFnType::Binary),
                precedence: Precedence::Equality,
            },
            TokenType::Greater => ParseRule {
                prefix: None,
                infix: Some(ParseFnType::Binary),
                precedence: Precedence::Ccomparison,
            },
            TokenType::GreaterEqual => ParseRule {
                prefix: None,
                infix: Some(ParseFnType::Binary),
                precedence: Precedence::Ccomparison,
            },
            TokenType::Less => ParseRule {
                prefix: None,
                infix: Some(ParseFnType::Binary),
                precedence: Precedence::Ccomparison,
            },
            TokenType::LessEqual => ParseRule {
                prefix: None,
                infix: Some(ParseFnType::Binary),
                precedence: Precedence::Ccomparison,
            },
            TokenType::Identifier => none_rule,
            TokenType::String => none_rule,
            TokenType::Number => ParseRule {
                prefix: Some(ParseFnType::Number),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::And => none_rule,
            TokenType::Class => none_rule,
            TokenType::Else => none_rule,
            TokenType::False => ParseRule {
                prefix: Some(ParseFnType::Literal),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Function => none_rule,
            TokenType::For => none_rule,
            TokenType::If => none_rule,
            TokenType::Nil => ParseRule {
                prefix: Some(ParseFnType::Literal),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Or => none_rule,
            TokenType::Print => none_rule,
            TokenType::Return => none_rule,
            TokenType::Super => none_rule,
            TokenType::This => none_rule,
            TokenType::True => ParseRule {
                prefix: Some(ParseFnType::Literal),
                infix: None,
                precedence: Precedence::None,
            },
            TokenType::Var => none_rule,
            TokenType::While => none_rule,
            TokenType::Eof => none_rule,
        },
        Err(e) => panic!("{}", e),
    })
});

pub fn get_parse_rule(typ: TokenType) -> ParseRule {
    return PARSE_RULES[typ as usize];
}
