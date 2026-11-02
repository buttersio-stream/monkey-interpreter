use std::fmt::Debug;

use crate::{
    ast::{
        Expression, ExpressionStatement, Identifier, InfixExpression, IntegerLiteral, LetStatement,
        PrefixExpression, Program, ReturnStatement, Statement,
    },
    lexer::Lexer,
    parser::Precedence::{Equals, LessGreater, Lowest, Prefix, Product, Sum},
    token::{
        Token,
        TokenType::{
            self, Assign, Asterisk, Bang, Eof, Eq, Gt, Ident, Int, Let, Lt, Minus, NotEq, Plus,
            Return, Semicolon, Slash,
        },
    },
};

#[derive(PartialEq, PartialOrd)]
enum Precedence {
    Lowest,
    Equals,
    LessGreater,
    Sum,
    Product,
    Prefix,
    _Call,
}

fn get_precedence(operator: &TokenType) -> Option<Precedence> {
    match operator {
        Eq | NotEq => Some(Equals),
        Lt | Gt => Some(LessGreater),
        Plus | Minus => Some(Sum),
        Slash | Asterisk => Some(Product),
        _ => None,
    }
}

#[derive(Default, Debug)]
pub struct Parser {
    l: Lexer,
    cur_token: Token,
    peek_token: Token,
    errors: Vec<String>,
}

impl Parser {
    pub fn new(l: Lexer) -> Self {
        let mut p = Parser {
            l,
            ..Self::default()
        };

        p.next_token();
        p.next_token();
        p
    }

    fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.l.next_token();
    }

    pub fn parse_program(&mut self) -> Program {
        let mut program = Program {
            statements: Vec::new(),
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

    fn parse_statement(&mut self) -> Option<Box<dyn Statement>> {
        match self.cur_token.token_type {
            Let => self.parse_let_statement(),
            Return => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Box<dyn Expression>> {
        // Parse possible prefix expression
        let mut exp = match self.cur_token.token_type {
            Ident => self.parse_identifier(),
            Int => self.parse_integer_literal(),
            Bang | Minus => self.parse_prefix_expression(),
            unknown => {
                self.errors
                    .push(format!("no prefix parse function for {:?}", unknown));
                None
            }
        };

        // Returns early if exp is None
        exp.as_ref()?;

        while !self.peek_token_is(Semicolon) && precedence < self.peek_precedence() {
            let infix = match self.peek_token.token_type {
                Plus | Minus | Slash | Asterisk | Eq | NotEq | Lt | Gt => {
                    self.next_token();
                    self.parse_infix_expression(exp.clone())
                }
                _ => None,
            };

            if infix.is_none() {
                break;
            } else {
                exp = infix;
            }
        }

        exp
    }

    fn parse_identifier(&self) -> Option<Box<dyn Expression>> {
        Some(Box::new(Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        }))
    }

    fn parse_integer_literal(&self) -> Option<Box<dyn Expression>> {
        let possible_int = self.cur_token.literal.parse::<i64>();

        if let Ok(value) = possible_int {
            Some(Box::new(IntegerLiteral {
                token: self.cur_token.clone(),
                value,
            }))
        } else {
            None
        }
    }

    fn parse_prefix_expression(&mut self) -> Option<Box<dyn Expression>> {
        let mut expression = PrefixExpression {
            token: self.cur_token.clone(),
            operator: self.cur_token.literal.clone(),
            ..PrefixExpression::default()
        };

        self.next_token();
        expression.right = self.parse_expression(Prefix);

        Some(Box::new(expression))
    }

    fn parse_infix_expression(
        &mut self,
        left: Option<Box<dyn Expression>>,
    ) -> Option<Box<dyn Expression>> {
        let mut expression = InfixExpression {
            token: self.cur_token.clone(),
            operator: self.cur_token.literal.clone(),
            left,
            ..InfixExpression::default()
        };

        let precedence = self.cur_precedence();
        self.next_token();
        expression.right = self.parse_expression(precedence);

        Some(Box::new(expression))
    }

    fn parse_expression_statement(&mut self) -> Option<Box<dyn Statement>> {
        let mut stmt = ExpressionStatement {
            token: self.cur_token.clone(),
            ..ExpressionStatement::default()
        };
        stmt.expression = self.parse_expression(Lowest);

        if self.peek_token_is(Semicolon) {
            self.next_token();
        }

        Some(Box::new(stmt))
    }

    fn parse_return_statement(&mut self) -> Option<Box<dyn Statement>> {
        let cur_token = self.cur_token.clone();

        self.next_token();

        // TODO: We're skipping the expression until we
        // encounter a semicolon
        loop {
            if self.cur_token_is(Semicolon) {
                break;
            } else {
                self.next_token();
            }
        }

        Some(Box::new(ReturnStatement {
            token: cur_token,
            return_value: None,
        }))
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
            value: literal,
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

        Some(Box::new(LetStatement {
            token: cur_token,
            name,
            value: None,
        }))
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
        let msg = format!(
            "expected next token to be {:?}, got {:?} instead",
            t, self.peek_token.token_type
        );
        self.errors.push(msg);
    }

    fn peek_precedence(&self) -> Precedence {
        match get_precedence(&self.peek_token.token_type) {
            Some(precedence) => precedence,
            None => Lowest,
        }
    }

    fn cur_precedence(&self) -> Precedence {
        match get_precedence(&self.cur_token.token_type) {
            Some(precedence) => precedence,
            None => Lowest,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{
            Expression, ExpressionStatement, Identifier, InfixExpression, IntegerLiteral,
            LetStatement, Node, PrefixExpression, ReturnStatement, Statement,
        },
        lexer::Lexer,
        parser::Parser,
    };

    #[test]
    fn test_return_statements() {
        let input = String::from(
            "
            return 5;
            return 10;
            return 993322;
        ",
        );

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program();
        check_parser_errors(&parser);

        assert_eq!(program.statements.len(), 3);

        for statement in program.statements {
            if let Some(s) = statement.as_any().downcast_ref::<ReturnStatement>() {
                assert_eq!(s.token_literal(), "return");
            } else {
                panic!("assertion failed: matched unexpected type {:?}", statement)
            };
        }
    }

    #[test]
    fn test_let_statements() {
        let input = String::from(
            "
            let x = 5;
            let y = 10;
            let foobar = 838383;
        ",
        );

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program();
        check_parser_errors(&parser);

        assert_eq!(program.statements.len(), 3);

        let tests = ["x".to_string(), "y".to_string(), "foobar".to_string()];

        for (i, expected_identifier) in tests.iter().enumerate() {
            test_let_statement(&(*program.statements[i]), expected_identifier);
        }
    }

    #[test]
    fn test_parsing_infix_expressions() {
        let tests = [
            ("5 + 5".to_string(), 5, "+", 5),
            ("5 - 5".to_string(), 5, "-", 5),
            ("5 * 5".to_string(), 5, "*", 5),
            ("5 / 5".to_string(), 5, "/", 5),
            ("5 > 5".to_string(), 5, ">", 5),
            ("5 < 5".to_string(), 5, "<", 5),
            ("5 == 5".to_string(), 5, "==", 5),
            ("5 != 5".to_string(), 5, "!=", 5),
        ];

        for (input, left_value, operator, right_value) in tests {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program();
            check_parser_errors(&parser);

            assert_eq!(program.statements.len(), 1);

            let possible_expression_statement = &program.statements[0];
            let stmt = possible_expression_statement
                .as_any()
                .downcast_ref::<ExpressionStatement>()
                .unwrap_or_else(|| {
                    panic!(
                        "assertion failed: matched unexpected type {:?}",
                        possible_expression_statement
                    )
                });

            if let Some(value) = &stmt.expression {
                let possible_infix_expression = value.as_any().downcast_ref::<InfixExpression>();

                if let Some(exp) = possible_infix_expression {
                    test_integer_literal(&exp.left, left_value);
                    assert_eq!(exp.operator, operator);
                    test_integer_literal(&exp.right, right_value);
                } else {
                    panic!(
                        "assertion_failed: expression not InfixExpression, got {:?}",
                        possible_infix_expression
                    )
                }
            } else {
                panic!("assertion_failed: no expression");
            }
        }
    }

    #[test]
    fn test_parsing_prefix_expressions() {
        let tests = [("!5;".to_string(), "!", 5), ("-15".to_string(), "-", 15)];

        for (input, operator, integer_value) in tests {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program();
            check_parser_errors(&parser);

            assert_eq!(program.statements.len(), 1);

            let possible_expression_statement = &program.statements[0];
            let stmt = possible_expression_statement
                .as_any()
                .downcast_ref::<ExpressionStatement>()
                .unwrap_or_else(|| {
                    panic!(
                        "assertion_failed: matched unexpected type {:?}",
                        possible_expression_statement
                    )
                });

            if let Some(ident_value) = &stmt.expression {
                let possible_prefix_expression =
                    ident_value.as_any().downcast_ref::<PrefixExpression>();

                if let Some(exp) = possible_prefix_expression {
                    assert_eq!(exp.operator, operator);
                    test_integer_literal(&exp.right, integer_value);
                } else {
                    panic!(
                        "assertion_failed: expression not PrefixExpression, got {:?}",
                        possible_prefix_expression
                    )
                }
            } else {
                panic!("assertion_failed: no expression");
            }
        }
    }

    fn test_integer_literal(possible_expression: &Option<Box<dyn Expression>>, expected: i64) {
        if let Some(expression) = possible_expression {
            let integer_literal = expression
                .as_any()
                .downcast_ref::<IntegerLiteral>()
                .unwrap_or_else(|| {
                    panic!("assertion_failed: matched unexpected_type {:?}", expression)
                });

            assert_eq!(integer_literal.value, expected);
            assert_eq!(integer_literal.token_literal(), format!("{}", expected));
        } else {
            panic!("assertion_failed: no expression")
        }
    }

    fn check_parser_errors(p: &Parser) {
        if p.errors.is_empty() {
            return;
        }

        let count_message = if p.errors.len() == 1 {
            "parser has 1 error\n".to_string()
        } else {
            format!("parser has {} errors\n", p.errors.len())
        };

        let error_messages = p
            .errors
            .iter()
            .map(|s| format!(" {}", s))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(
            p.errors.is_empty(),
            "assertion failed: {count_message}{error_messages}\n"
        );
    }

    fn test_let_statement(statement: &dyn Statement, expected_identifier: &String) {
        if let Some(s) = statement.as_any().downcast_ref::<LetStatement>() {
            assert_eq!(s.token_literal(), "let");
            assert_eq!(&s.name.value, expected_identifier);
            assert_eq!(&s.name.token_literal(), expected_identifier);
        } else {
            panic!("assertion failed: matched unexpected type {:?}", statement)
        };
    }

    #[test]
    fn test_identifier_expression() {
        let input = String::from("foobar;");

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        check_parser_errors(&parser);

        assert_eq!(program.statements.len(), 1);

        let possible_expression_statement = &program.statements[0];
        let stmt = possible_expression_statement
            .as_any()
            .downcast_ref::<ExpressionStatement>()
            .unwrap_or_else(|| {
                panic!(
                    "assertion_failed: matched unexpected type {:?}",
                    possible_expression_statement
                )
            });

        if let Some(ident_value) = &stmt.expression {
            let possible_identifier_expression = ident_value.as_any().downcast_ref::<Identifier>();

            if let Some(ident) = possible_identifier_expression {
                assert_eq!(ident.value, "foobar");
                assert_eq!(ident.token_literal(), "foobar");
            } else {
                panic!(
                    "assertion_failed: expression not Identifier, got {:?}",
                    possible_identifier_expression
                )
            }
        } else {
            panic!("assertion_failed: no expression");
        }
    }

    #[test]
    fn test_integer_literal_expression() {
        let input = String::from("5;");

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        check_parser_errors(&parser);

        assert_eq!(program.statements.len(), 1);

        let possible_expression_statement = &program.statements[0];
        let stmt = possible_expression_statement
            .as_any()
            .downcast_ref::<ExpressionStatement>()
            .unwrap_or_else(|| {
                panic!(
                    "assertion_failed: matched unexpected type {:?}",
                    possible_expression_statement
                )
            });

        if let Some(ident_value) = &stmt.expression {
            let possible_identifier_expression =
                ident_value.as_any().downcast_ref::<IntegerLiteral>();

            if let Some(ident) = possible_identifier_expression {
                assert_eq!(ident.value, 5);
                assert_eq!(ident.token_literal(), "5");
            } else {
                panic!(
                    "assertion_failed: expression not IntegerLiteral, got {:?}",
                    possible_identifier_expression
                )
            }
        } else {
            panic!("assertion_failed: no expression");
        }
    }
}
