use std::{any::Any, fmt::Debug};

use crate::{token::Token};

pub trait Node {
    fn token_literal(&self) -> String;
    fn to_string(&self) -> String;
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
    fn token_literal(&self) -> String {
        if !self.statements.is_empty() {
            self.statements[0].token_literal()
        } else {
            "".to_string()
        }
    }

    fn to_string(&self) -> String {
        let mut out = String::new();

        for s in &self.statements {
            out.push_str(&s.to_string());
        }

        out
    }

    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Debug)]
pub struct LetStatement {
    pub token: Token,
    pub name: Identifier,
    pub value: Option<Box<dyn Expression>>
}

impl Node for LetStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!("{} ", self.token_literal()));
        out.push_str(&self.name.to_string());
        out.push_str(" = ");

        if let Some(v) = &self.value {
            out.push_str(&v.to_string());
        }

        out.push(';');

        out
    }

    fn as_any(&self) -> &dyn Any { self }
}

impl Statement for LetStatement {
    fn statement_node(&self) {}
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

    fn to_string(&self) -> String {
        self.value.clone()
    }

    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Debug)]
pub struct ReturnStatement {
    pub token: Token,
    pub return_value: Option<Box<dyn Expression>>
}

impl Node for ReturnStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!("{} ", self.token_literal()));

        if let Some(rv) = &self.return_value {
            out.push_str(&rv.to_string());
        }

        out.push(';');
        out
    }

    fn as_any(&self) -> &dyn Any { self }
}

impl Statement for ReturnStatement {
    fn statement_node(&self) {}
}

#[derive(Debug)]
pub struct ExpressionStatement {
    pub token: Token,
    pub expression: Option<Box<dyn Expression>>
}

impl Node for ExpressionStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn to_string(&self) -> String {
        if let Some(e) = &self.expression {
            e.to_string()
        } else {
            "".to_string()
        }
    }

    fn as_any(&self) -> &dyn Any { self }
}

impl Statement for ExpressionStatement {
    fn statement_node(&self) {}
}

#[cfg(test)]
mod tests {
    use crate::{ast::{Identifier, LetStatement, Node, Program}, token::{Token, TokenType::{Ident, Let}}};

    #[test]
    fn test_to_string() {
        let program = Program {
            statements: vec![
                Box::new(
                    LetStatement {
                        token: Token {
                            token_type: Let,
                            literal: "let".to_string()
                        },
                        name: Identifier { 
                            token: Token { 
                                token_type:Ident, 
                                literal: "myVar".to_string()
                            }, 
                            value: "myVar".to_string()
                        },
                        value: Some(
                            Box::new(
                                Identifier {
                                    token: Token {
                                        token_type: Ident,
                                        literal: "anotherVar".to_string()
                                    },
                                    value: "anotherVar".to_string()
                                }
                            )
                        )
                    }
                )
            ]
        };

        assert_eq!("let myVar = anotherVar;", program.to_string())
    }
}