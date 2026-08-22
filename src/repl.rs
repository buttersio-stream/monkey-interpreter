use std::io::{self, BufRead, Read, Write};

use crate::{lexer::Lexer, token::TokenType::Eof};

const PROMPT: &str = ">> ";

pub fn start<R: Read + BufRead, W: Write>(mut input_stream: R, mut output_stream: W ) -> io::Result<()> {
    loop { 
        print!("{}", PROMPT);

        output_stream
            .flush()
            .expect("Failed to flush to stdout");
        
        let mut input = String::new();

        input_stream
            .read_line(&mut input)
            .expect("Failed to read line");

        let mut lexer = Lexer::new(input);

        loop {
            let token = lexer.next_token();

            if token.token_type == Eof {
                break;
            } else {
                println!("{:?}", token)
            }
        }
    }
}