use std::{any::Any, fmt::Debug};

use crate::token::Token;

pub trait Node {
    fn token_literal(&self) -> String;
    fn as_any(&self) -> &dyn Any;
}


pub trait Statement: Node + Debug {
    fn statement_node(&self);
}


pub trait Expression: Node + Debug {
    fn expression_node(&self);
}

pub struct Program {
    pub statements: Vec<Box<dyn Statement>>
}

impl Node for Program {
    fn as_any(&self) -> &dyn Any { self }

    fn token_literal(&self) -> String {
        if self.statements.len() > 0 {
            self.statements[0].token_literal()
        } else {
            "".to_string()
        }
    }
}

#[derive(Debug)]
pub struct LetStatement {
    pub token: Token,
    pub name: Identifier,
    pub value: Box<Option<i32>>
}

impl Statement for LetStatement {
    fn statement_node(&self) {}
}

impl Node for LetStatement {
    fn as_any(&self) -> &dyn Any { self }

    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

#[derive(Default, Debug)]
pub struct Identifier {
    pub token: Token,
    pub value: String
}

impl Expression for Identifier {
    fn expression_node(&self) {}
}

impl Node for Identifier {
    fn as_any(&self) -> &dyn Any { self }

    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}