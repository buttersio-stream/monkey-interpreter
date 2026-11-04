use std::fmt::Debug;

use crate::{
    ast::{
        BooleanLiteral, Expression, ExpressionStatement, Identifier, InfixExpression,
        IntegerLiteral, LetStatement, PrefixExpression, Program, ReturnStatement, Statement,
    },
    lexer::Lexer,
    parser::Precedence::{Equals, LessGreater, Lowest, Prefix, Product, Sum},
    token::{
        Token,
        TokenType::{
            self, Assign, Asterisk, Bang, Eof, Eq, False, Gt, Ident, Int, LParen, Let, Lt, Minus,
            NotEq, Plus, RParen, Return, Semicolon, Slash, True,
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
            True | False => self.parse_boolean_literal(),
            Bang | Minus => self.parse_prefix_expression(),
            LParen => self.parse_grouped_expression(),
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

    fn parse_boolean_literal(&self) -> Option<Box<dyn Expression>> {
        let possible_bool = self.cur_token.literal.parse::<bool>();

        if let Ok(value) = possible_bool {
            Some(Box::new(BooleanLiteral {
                token: self.cur_token.clone(),
                value,
            }))
        } else {
            None
        }
    }

    fn parse_grouped_expression(&mut self) -> Option<Box<dyn Expression>> {
        self.next_token();

        let exp = self.parse_expression(Lowest);

        if !self.expect_peek(RParen) {
            return None;
        }

        exp
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
            BooleanLiteral, Expression, ExpressionStatement, Identifier, InfixExpression,
            IntegerLiteral, LetStatement, Node, PrefixExpression, ReturnStatement, Statement,
        },
        lexer::Lexer,
        parser::{
            Parser,
            tests::Literal::{_Text, Boolean, Number},
        },
    };

    enum Literal {
        _Text(String),
        Number(i64),
        Boolean(bool),
    }

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
    fn test_operator_precedence_parsing() {
        let tests = [
            ("-a * b".to_string(), "((-a) * b)".to_string()),
            ("!-a".to_string(), "(!(-a))".to_string()),
            ("a + b + c".to_string(), "((a + b) + c)".to_string()),
            ("a + b - c".to_string(), "((a + b) - c)".to_string()),
            ("a * b * c".to_string(), "((a * b) * c)".to_string()),
            ("a * b / c".to_string(), "((a * b) / c)".to_string()),
            ("a + b / c".to_string(), "(a + (b / c))".to_string()),
            (
                "a + b * c + d / e - f".to_string(),
                "(((a + (b * c)) + (d / e)) - f)".to_string(),
            ),
            ("3 + 4; -5 * 5".to_string(), "(3 + 4)((-5) * 5)".to_string()),
            (
                "5 > 4 == 3 < 4".to_string(),
                "((5 > 4) == (3 < 4))".to_string(),
            ),
            (
                "5 < 4 != 3 > 4".to_string(),
                "((5 < 4) != (3 > 4))".to_string(),
            ),
            (
                "3 + 4 * 5 == 3 * 1 + 4 * 5".to_string(),
                "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))".to_string(),
            ),
            ("true".to_string(), "true".to_string()),
            ("false".to_string(), "false".to_string()),
            (
                "3 > 5 == false".to_string(),
                "((3 > 5) == false)".to_string(),
            ),
            ("3 < 5 == true".to_string(), "((3 < 5) == true)".to_string()),
            (
                "1 + (2 + 3) + 4".to_string(),
                "((1 + (2 + 3)) + 4)".to_string(),
            ),
            ("(5 + 5) * 2".to_string(), "((5 + 5) * 2)".to_string()),
            ("2 / (5 + 5)".to_string(), "(2 / (5 + 5))".to_string()),
            ("-(5 + 5)".to_string(), "(-(5 + 5))".to_string()),
            (
                "!(true == true)".to_string(),
                "(!(true == true))".to_string(),
            ),
        ];

        for (input, expected) in tests {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program();
            check_parser_errors(&parser);

            let actual = program.to_string();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_parsing_infix_expressions() {
        let tests = [
            (
                "5 + 5".to_string(),
                Literal::Number(5),
                "+",
                Literal::Number(5),
            ),
            (
                "5 - 5".to_string(),
                Literal::Number(5),
                "-",
                Literal::Number(5),
            ),
            (
                "5 * 5".to_string(),
                Literal::Number(5),
                "*",
                Literal::Number(5),
            ),
            (
                "5 / 5".to_string(),
                Literal::Number(5),
                "/",
                Literal::Number(5),
            ),
            (
                "5 > 5".to_string(),
                Literal::Number(5),
                ">",
                Literal::Number(5),
            ),
            (
                "5 < 5".to_string(),
                Literal::Number(5),
                "<",
                Literal::Number(5),
            ),
            (
                "5 == 5".to_string(),
                Literal::Number(5),
                "==",
                Literal::Number(5),
            ),
            (
                "5 != 5".to_string(),
                Literal::Number(5),
                "!=",
                Literal::Number(5),
            ),
            (
                "true == true".to_string(),
                Literal::Boolean(true),
                "==",
                Literal::Boolean(true),
            ),
            (
                "true != false".to_string(),
                Literal::Boolean(true),
                "!=",
                Literal::Boolean(false),
            ),
            (
                "false == false".to_string(),
                Literal::Boolean(false),
                "==",
                Literal::Boolean(false),
            ),
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

            test_infix_expression(&stmt.expression, left_value, operator, right_value).unwrap();
        }
    }

    #[test]
    fn test_parsing_prefix_expressions() {
        let tests = [
            ("!5;".to_string(), "!", Literal::Number(5)),
            ("-15".to_string(), "-", Literal::Number(15)),
            ("!true".to_string(), "!", Literal::Boolean(true)),
            ("!false".to_string(), "!", Literal::Boolean(false)),
        ];

        for (input, operator, value) in tests {
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
                    test_literal_expression(&exp.right, value).unwrap();
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

    fn test_integer_literal(
        possible_expression: &Option<Box<dyn Expression>>,
        expected: i64,
    ) -> Result<(), String> {
        if let Some(expression) = possible_expression {
            if let Some(integer_literal) = expression.as_any().downcast_ref::<IntegerLiteral>() {
                if integer_literal.value != expected {
                    return Err(format!(
                        "Expected {} to equal {}",
                        integer_literal.value, expected
                    ));
                }

                if integer_literal.token_literal() != format!("{}", expected) {
                    return Err(format!(
                        "Expected {} to equal {}",
                        integer_literal.token_literal(),
                        expected
                    ));
                }

                Ok(())
            } else {
                Err(format!(
                    "assertion_failed: matched unexpected_type {:?}",
                    expression
                ))
            }
        } else {
            Err("assertion_failed: no expression".to_string())
        }
    }

    fn test_boolean_literal(
        possible_expression: &Option<Box<dyn Expression>>,
        expected: bool,
    ) -> Result<(), String> {
        if let Some(expression) = possible_expression {
            if let Some(boolean_literal) = expression.as_any().downcast_ref::<BooleanLiteral>() {
                if boolean_literal.value != expected {
                    return Err(format!(
                        "Expected {} to equal {}",
                        boolean_literal.value, expected
                    ));
                }

                if boolean_literal.token_literal() != format!("{}", expected) {
                    return Err(format!(
                        "Expected {} to equal {}",
                        boolean_literal.token_literal(),
                        expected
                    ));
                }

                Ok(())
            } else {
                Err(format!(
                    "assertion_failed: matched unexpected_type {:?}",
                    expression
                ))
            }
        } else {
            Err("assertion_failed: no expression".to_string())
        }
    }

    fn test_identifier(
        possible_expression: &Option<Box<dyn Expression>>,
        expected: String,
    ) -> Result<(), String> {
        if let Some(expression) = possible_expression {
            if let Some(identifier) = expression.as_any().downcast_ref::<Identifier>() {
                if identifier.value != expected {
                    return Err(format!(
                        "Expected {} to equal {}",
                        identifier.value, expected
                    ));
                }

                if identifier.token_literal() != expected {
                    return Err(format!(
                        "Expected {} to equal {}",
                        identifier.token_literal(),
                        expected
                    ));
                }

                Ok(())
            } else {
                Err(format!(
                    "assertion_failed: matched unexpected_type {:?}",
                    expression
                ))
            }
        } else {
            Err("assertion_failed: no expression".to_string())
        }
    }

    fn test_literal_expression(
        possible_expression: &Option<Box<dyn Expression>>,
        expected: Literal,
    ) -> Result<(), String> {
        match expected {
            _Text(value) => test_identifier(possible_expression, value),
            Number(value) => test_integer_literal(possible_expression, value),
            Boolean(value) => test_boolean_literal(possible_expression, value),
        }
    }

    fn test_infix_expression(
        possible_expression: &Option<Box<dyn Expression>>,
        left: Literal,
        operator: &str,
        right: Literal,
    ) -> Result<(), String> {
        if let Some(expression) = possible_expression {
            if let Some(infix) = expression.as_any().downcast_ref::<InfixExpression>() {
                test_literal_expression(&infix.left, left)?;
                if infix.operator != operator {
                    return Err(format!("Expected {} to equal {}", infix.operator, operator));
                }
                test_literal_expression(&infix.right, right)?;

                Ok(())
            } else {
                Err(format!(
                    "assertion_failed: matched unexpected_type {:?}",
                    expression
                ))
            }
        } else {
            Err("assertion_failed: no expression".to_string())
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

        test_identifier(&stmt.expression, "foobar".to_string()).unwrap();
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

        test_integer_literal(&stmt.expression, 5).unwrap();
    }

    #[test]
    fn test_boolean_expression() {
        let tests = [
            (String::from("true;"), true),
            (String::from("false;"), false),
        ];

        for (input, expected) in tests {
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

            test_boolean_literal(&stmt.expression, expected).unwrap();
        }
    }
}
