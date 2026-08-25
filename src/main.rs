pub mod token;
pub mod lexer;
pub mod repl;
pub mod ast;
pub mod parser;

use std::io;

fn main() -> io::Result<()>{
    let stdin = io::stdin();
    let stdout = io::stdout();

    let stdin_lock = stdin.lock();
    let stdout_lock = stdout.lock();

    println!("Hello! This is the Monkey programming language!");
    repl::start(stdin_lock, stdout_lock)
}

