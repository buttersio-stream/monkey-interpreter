use crate::token::TokenType::{Eof};

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: String
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TokenType {
    Illegal,
    Eof,
    Ident,
    Int,
    Assign,
    Plus,
    Minus,
    Bang,
    Asterisk,
    Slash,
    Lt,
    Gt,
    Comma,
    Semicolon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Function,
    Let,
    True,
    False,
    If,
    Else,
    Return,
    Eq,
    NotEq
}

impl Default for TokenType {
    fn default() -> Self {
        Eof
    }
}