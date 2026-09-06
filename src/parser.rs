use crate::{ast::{Identifier, Program, Statement}, lexer::Lexer, token::{Token, TokenType::{self, Assign, Eof, Ident, Let, Semicolon}}};

#[derive(Default, Debug)]
struct Parser {
    l: Lexer,
    cur_token: Token,
    peek_token: Token,
    errors: Vec<String>
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
            if self.cur_token_is(Eof) {
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

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.cur_token.token_type {
            Let => self.parse_let_statement(),
            _ => None
        }
    }

    fn parse_let_statement(&mut self) -> Option<Statement> {
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
            Statement::LetStatement {
                token: cur_token,
                name: name,
                value: Box::new(None)
            }
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
            self.peek_error(t);
            false
        }
    }

    fn peek_error(&mut self, t: TokenType) {
        let msg = format!("expected next token to be {:?}, got {:?} instead", t, self.peek_token.token_type);
        self.errors.push(msg);
    }
}

#[cfg(test)]
mod tests {
    use crate::{ast::{Node, Statement}, lexer::Lexer, parser::Parser};

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
        check_parser_errors(&parser);

        assert_eq!(program.statements.len(), 3);

        let tests = [
            "x".to_string(), 
            "y".to_string(), 
            "foobar".to_string()];

        for (i, expected_identifier) in tests.iter().enumerate() {
            test_let_statement(&program.statements[i], expected_identifier);
        }
    }

    fn check_parser_errors(p: &Parser) {
        if p.errors.len() == 0 {
            return
        }

        let count_message = format!("parser has {} errors\n", p.errors.len());
        let error_messages = (&p.errors)
            .into_iter()
            .map(|s| format!(" {}", s))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(p.errors.len() == 0, "assertion failed: {count_message}{error_messages}\n");
    }

    fn test_let_statement(statement: &Statement, expected_identifier: &String) {
        assert_eq!(statement.token_literal(), "let");

        assert!(matches!(statement, Statement::LetStatement { .. }));

        if let Statement::LetStatement { name, .. } =  statement{
            assert_eq!(&name.value, expected_identifier);
            assert_eq!(&name.token_literal(), expected_identifier);
        }
    }
}