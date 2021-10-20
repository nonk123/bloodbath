use crate::interpreter::Bloodbath;
use crate::interpreter::ParserError;
use crate::reader::ReaderError;
use std::io::Write;

mod builtins;
mod interpreter;
mod object;
mod reader;

fn main() {
    let mut bloodbath = Bloodbath::new();

    println!("Welcome to the Bloodbath REPL!");
    println!("Enter an expression to evaluate it. Type \"quit\" to exit.");

    loop {
        print!("> ");

        if let Err(err) = std::io::stdout().flush() {
            println!("IO error: {}", err);
            std::process::exit(1);
        }

        let mut line = String::new();

        if let Err(err) = std::io::stdin().read_line(&mut line) {
            println!("IO error: {}", err);
            std::process::exit(1);
        }

        // Strip line ending.
        if line.ends_with('\n') {
            line.pop();

            if line.ends_with('\r') {
                line.pop();
            }
        }

        if line == "quit".to_string() {
            println!("Goodbye!");
            break;
        }

        match bloodbath.eval(line) {
            Ok(object) => println!("{:?}", object),
            Err(ParserError::ReadingFailed(err)) => match err {
                ReaderError::EoF => println!("Unexpected end of file"),
                ReaderError::UnexpectedCharacter(bad_char) => {
                    println!("Unexpected character: '{}'", bad_char)
                }
            },
            Err(err) => println!("{:?}", err),
        }
    }
}
