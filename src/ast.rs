use std::{fmt::Debug};

use crate::{token::Token};

pub trait Node {
    fn token_literal(&self) -> String;
}

#[derive(Debug)]
pub enum Statement {
    LetStatement {
        token: Token,
        name: Identifier,
        value: Box<Option<i32>>
    },
    ReturnStatement {
        token: Token,
        return_value: Box<Option<i32>>
    }
}


pub trait Expression: Node + Debug {
    fn expression_node(&self);
}

pub struct Program {
    pub statements: Vec<Statement>
}

impl Node for Program {
    fn token_literal(&self) -> String {
        if !self.statements.is_empty() {
            self.statements[0].token_literal()
        } else {
            "".to_string()
        }
    }
}

#[derive(Debug)]
pub struct LetStatementData {
    pub token: Token,
    pub name: Identifier,
    pub value: Box<Option<i32>>
}

impl Node for Statement {
    fn token_literal(&self) -> String {
        match self {
            Statement::LetStatement { token, .. } => {
                token.literal.clone()
            },
            Statement::ReturnStatement { token, .. } => {
                token.literal.clone()
            }
        }
    }
}

#[derive(Debug)]
pub struct Identifier {
    pub token: Token,
    pub value: String
}

impl Expression for Identifier {
    fn expression_node(&self) {}
}

impl Node for Identifier {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}