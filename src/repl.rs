use std::io::{self, BufRead, Read, Write};

use crate::{ast::Node, lexer::Lexer, parser::Parser};

const PROMPT: &str = ">> ";

pub fn start<R: Read + BufRead, W: Write>(
    mut input_stream: R,
    mut output_stream: W,
) -> io::Result<()> {
    loop {
        print!("{}", PROMPT);

        output_stream.flush().expect("Failed to flush to stdout");

        let mut input = String::new();

        input_stream
            .read_line(&mut input)
            .expect("Failed to read line");

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        println!("{}", parser.parse_program().to_string())
    }
}
