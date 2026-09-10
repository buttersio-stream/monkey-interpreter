use crate::token::{
    Token,
    TokenType::{
        self, Assign, Asterisk, Bang, Comma, Else, Eof, Eq, False, Function, Gt, Ident, If,
        Illegal, Int, LBrace, LParen, Let, Lt, Minus, NotEq, Plus, RBrace, RParen, Return,
        Semicolon, Slash, True,
    },
};

#[derive(Default, Debug)]
pub struct Lexer {
    input: String,
    position: usize,
    read_position: usize,
    ch: char,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let mut lexer = Lexer {
            input,
            ..Self::default()
        };
        lexer.next_token();
        lexer
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.chars().count() {
            self.ch = '\0'
        } else {
            self.ch = self.input.chars().nth(self.read_position).unwrap();
        };
        self.position = self.read_position;
        self.read_position += 1;
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let mut already_read = false;

        let token = match self.ch {
            '=' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token {
                        token_type: Eq,
                        literal: "==".to_string(),
                    }
                } else {
                    Token {
                        token_type: Assign,
                        literal: "=".to_string(),
                    }
                }
            }
            ';' => Token {
                token_type: Semicolon,
                literal: ";".to_string(),
            },
            '(' => Token {
                token_type: LParen,
                literal: "(".to_string(),
            },
            ')' => Token {
                token_type: RParen,
                literal: ")".to_string(),
            },
            ',' => Token {
                token_type: Comma,
                literal: ",".to_string(),
            },
            '+' => Token {
                token_type: Plus,
                literal: "+".to_string(),
            },
            '{' => Token {
                token_type: LBrace,
                literal: "{".to_string(),
            },
            '}' => Token {
                token_type: RBrace,
                literal: "}".to_string(),
            },
            '-' => Token {
                token_type: Minus,
                literal: "-".to_string(),
            },
            '!' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token {
                        token_type: NotEq,
                        literal: "!=".to_string(),
                    }
                } else {
                    Token {
                        token_type: Bang,
                        literal: "!".to_string(),
                    }
                }
            }
            '*' => Token {
                token_type: Asterisk,
                literal: "*".to_string(),
            },
            '/' => Token {
                token_type: Slash,
                literal: "/".to_string(),
            },
            '<' => Token {
                token_type: Lt,
                literal: "<".to_string(),
            },
            '>' => Token {
                token_type: Gt,
                literal: ">".to_string(),
            },
            '\0' => Token {
                token_type: Eof,
                literal: "".to_string(),
            },
            _ => {
                if is_letter(self.ch) {
                    already_read = true;
                    let ident = self.read_identifier();

                    Token {
                        token_type: lookup_ident(&ident),
                        literal: ident,
                    }
                } else if is_digit(self.ch) {
                    already_read = true;
                    Token {
                        token_type: Int,
                        literal: self.read_number(),
                    }
                } else {
                    Token {
                        token_type: Illegal,
                        literal: self.ch.to_string(),
                    }
                }
            }
        };

        if !already_read {
            self.read_char();
        }

        token
    }

    fn read_identifier(&mut self) -> String {
        let position = self.position;
        while is_letter(self.ch) {
            self.read_char();
        }

        self.input[position..self.position].to_string()
    }

    fn read_number(&mut self) -> String {
        let position = self.position;
        while is_digit(self.ch) {
            self.read_char();
        }

        self.input[position..self.position].to_string()
    }

    fn skip_whitespace(&mut self) {
        while self.ch == ' ' || self.ch == '\t' || self.ch == '\n' || self.ch == '\r' {
            self.read_char();
        }
    }

    fn peek_char(&self) -> char {
        if self.read_position >= self.input.chars().count() {
            '\0'
        } else {
            self.input.chars().nth(self.read_position).unwrap()
        }
    }
}

const KEYWORDS: [(&str, TokenType); 7] = [
    ("fn", Function),
    ("let", Let),
    ("true", True),
    ("false", False),
    ("if", If),
    ("else", Else),
    ("return", Return),
];

fn is_letter(ch: char) -> bool {
    ch.is_ascii_lowercase() || ch.is_ascii_uppercase() || ch == '_'
}

fn is_digit(ch: char) -> bool {
    ch.is_ascii_digit()
}

fn lookup_ident(ident: &str) -> TokenType {
    KEYWORDS
        .iter()
        .find(|&&(k, _)| k == ident)
        .map(|&(_, v)| v)
        .unwrap_or(Ident)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenType::{
        Assign, Bang, Comma, Eof, Function, Ident, Int, LBrace, LParen, Let, Plus, RBrace, RParen,
        Semicolon,
    };

    #[test]
    fn test_simple_operators() {
        let input = String::from("=+(){},;-!*/<>");

        let tests = vec![
            Token {
                token_type: Assign,
                literal: "=".to_string(),
            },
            Token {
                token_type: Plus,
                literal: "+".to_string(),
            },
            Token {
                token_type: LParen,
                literal: "(".to_string(),
            },
            Token {
                token_type: RParen,
                literal: ")".to_string(),
            },
            Token {
                token_type: LBrace,
                literal: "{".to_string(),
            },
            Token {
                token_type: RBrace,
                literal: "}".to_string(),
            },
            Token {
                token_type: Comma,
                literal: ",".to_string(),
            },
            Token {
                token_type: Semicolon,
                literal: ";".to_string(),
            },
            Token {
                token_type: Minus,
                literal: "-".to_string(),
            },
            Token {
                token_type: Bang,
                literal: "!".to_string(),
            },
            Token {
                token_type: Asterisk,
                literal: "*".to_string(),
            },
            Token {
                token_type: Slash,
                literal: "/".to_string(),
            },
            Token {
                token_type: Lt,
                literal: "<".to_string(),
            },
            Token {
                token_type: Gt,
                literal: ">".to_string(),
            },
            Token {
                token_type: Eof,
                literal: "".to_string(),
            },
        ];

        let mut lexer = Lexer::new(input);

        for expected in &tests {
            let token = lexer.next_token();
            assert_eq!(&token, expected);
        }
    }

    #[test]
    fn test_keywords() {
        let input = String::from(
            "
        let five = 5;
        let ten = 10;

        let add = fn(x, y) {
            x + y;
        };

        let result = add(five, ten);

        10 == 10;
        1 != 9;
        ",
        );

        let tests = vec![
            Token {
                token_type: Let,
                literal: "let".to_string(),
            },
            Token {
                token_type: Ident,
                literal: "five".to_string(),
            },
            Token {
                token_type: Assign,
                literal: "=".to_string(),
            },
            Token {
                token_type: Int,
                literal: "5".to_string(),
            },
            Token {
                token_type: Semicolon,
                literal: ";".to_string(),
            },
            Token {
                token_type: Let,
                literal: "let".to_string(),
            },
            Token {
                token_type: Ident,
                literal: "ten".to_string(),
            },
            Token {
                token_type: Assign,
                literal: "=".to_string(),
            },
            Token {
                token_type: Int,
                literal: "10".to_string(),
            },
            Token {
                token_type: Semicolon,
                literal: ";".to_string(),
            },
            Token {
                token_type: Let,
                literal: "let".to_string(),
            },
            Token {
                token_type: Ident,
                literal: "add".to_string(),
            },
            Token {
                token_type: Assign,
                literal: "=".to_string(),
            },
            Token {
                token_type: Function,
                literal: "fn".to_string(),
            },
            Token {
                token_type: LParen,
                literal: "(".to_string(),
            },
            Token {
                token_type: Ident,
                literal: "x".to_string(),
            },
            Token {
                token_type: Comma,
                literal: ",".to_string(),
            },
            Token {
                token_type: Ident,
                literal: "y".to_string(),
            },
            Token {
                token_type: RParen,
                literal: ")".to_string(),
            },
            Token {
                token_type: LBrace,
                literal: "{".to_string(),
            },
            Token {
                token_type: Ident,
                literal: "x".to_string(),
            },
            Token {
                token_type: Plus,
                literal: "+".to_string(),
            },
            Token {
                token_type: Ident,
                literal: "y".to_string(),
            },
            Token {
                token_type: Semicolon,
                literal: ";".to_string(),
            },
            Token {
                token_type: RBrace,
                literal: "}".to_string(),
            },
            Token {
                token_type: Semicolon,
                literal: ";".to_string(),
            },
            Token {
                token_type: Let,
                literal: "let".to_string(),
            },
            Token {
                token_type: Ident,
                literal: "result".to_string(),
            },
            Token {
                token_type: Assign,
                literal: "=".to_string(),
            },
            Token {
                token_type: Ident,
                literal: "add".to_string(),
            },
            Token {
                token_type: LParen,
                literal: "(".to_string(),
            },
            Token {
                token_type: Ident,
                literal: "five".to_string(),
            },
            Token {
                token_type: Comma,
                literal: ",".to_string(),
            },
            Token {
                token_type: Ident,
                literal: "ten".to_string(),
            },
            Token {
                token_type: RParen,
                literal: ")".to_string(),
            },
            Token {
                token_type: Semicolon,
                literal: ";".to_string(),
            },
            Token {
                token_type: Int,
                literal: "10".to_string(),
            },
            Token {
                token_type: Eq,
                literal: "==".to_string(),
            },
            Token {
                token_type: Int,
                literal: "10".to_string(),
            },
            Token {
                token_type: Semicolon,
                literal: ";".to_string(),
            },
            Token {
                token_type: Int,
                literal: "1".to_string(),
            },
            Token {
                token_type: NotEq,
                literal: "!=".to_string(),
            },
            Token {
                token_type: Int,
                literal: "9".to_string(),
            },
            Token {
                token_type: Semicolon,
                literal: ";".to_string(),
            },
            Token {
                token_type: Eof,
                literal: "".to_string(),
            },
        ];

        let mut lexer = Lexer::new(input);

        for expected in &tests {
            let token = lexer.next_token();
            assert_eq!(&token, expected);
        }
    }
}
