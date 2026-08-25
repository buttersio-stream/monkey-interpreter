use crate::{ast::{Identifier, LetStatement, Program, Statement}, lexer::Lexer, token::{Token, TokenType::{self, Assign, Eof, Ident, Let, Semicolon}}};

#[derive(Default, Debug)]
struct Parser {
    l: Lexer,
    cur_token: Token,
    peek_token: Token
}

impl Parser {
    fn new(l: Lexer) -> Self {
        let mut p = Self::default();
        p.l = l;

        p.next_token();
        p.next_token();
        p
    }

    fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.l.next_token();
    }

    fn parse_program(&mut self) -> Program {
        let mut program = Program {
            statements: Vec::new()
        };

        loop {
            if self.cur_token.token_type == Eof {
                break;
            } else {
                if let Some(stmt) = self.parse_statement() {
                    program.statements.push(stmt);
                }
            }
            self.next_token();
        }

        program
    }

    fn parse_statement(&mut self) -> Option<Box<dyn Statement>> {
        match self.cur_token.token_type {
            Let => self.parse_let_statement(),
            _ => None
        }
    }

    fn parse_let_statement(&mut self) -> Option<Box<dyn Statement>> {
        let cur_token = self.cur_token.clone();

        if !self.expect_peek(Ident) {
            return None;
        }

        let next_token = self.cur_token.clone();
        let literal = next_token.literal.clone();
        let name = Identifier {
            token: next_token,
            value: literal
        };

        if !self.expect_peek(Assign) {
            return None;
        }

        loop {
            if self.cur_token_is(Semicolon) {
                break;
            } else {
                self.next_token();
            }
        }

        Some(
            Box::new(
                LetStatement {
                    token: cur_token,
                    name: name,
                    value: Box::new(None)
                }
            )
        )
    }

    fn cur_token_is(&self, t: TokenType) -> bool {
        t == self.cur_token.token_type
    }

    fn peek_token_is(&self, t: TokenType) -> bool {
        t == self.peek_token.token_type
    }

    fn expect_peek(&mut self, t: TokenType) -> bool {
        if self.peek_token_is(t) {
            self.next_token();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ast::{LetStatement, Node, Statement}, lexer::Lexer, parser::Parser};

    #[test]
    fn test_let_statements() {
        let input = String::from("
            let x = 5;
            let y = 10;
            let foobar = 838383;
        ");

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program();

        assert_eq!(program.statements.len(), 3);

        let tests = [
            "x".to_string(), 
            "y".to_string(), 
            "foobar".to_string()];

        for (i, expected_identifier) in tests.iter().enumerate() {
            test_let_statement(&program.statements[i], expected_identifier);

        }
    }

    fn test_let_statement(statement: &Box<dyn Statement>, expected_identifier: &String) {
        let statement = &(*statement);
        assert_eq!(statement.token_literal(), "let");

        let option_let_statement = statement.as_any().downcast_ref::<LetStatement>();
        assert!(option_let_statement.is_some());

        if let Some(let_statement) = option_let_statement {
            assert_eq!(&let_statement.name.value, expected_identifier);
            assert_eq!(&let_statement.name.token_literal(), expected_identifier);
        }
    }
}