use crate::interpreter::Bloodbath;
use crate::interpreter::InterpreterError;
use crate::object::Object;
use crate::object::PrimitiveValue;
use crate::reader::ReaderError;
use std::io::Write;

mod interpreter;
mod object;
mod reader;

fn object_to_string(object: Object) -> String {
    match object {
        Object::Primitive(value) => match value {
            PrimitiveValue::Noop => "noop".into(),
            PrimitiveValue::Integer(value) => value.to_string(),
            PrimitiveValue::Float(value) => value.to_string(),
        },
    }
}

fn main() {
    let bloodbath = Bloodbath::new();

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
            Ok(object) => println!("{}", object_to_string(object)),
            Err(InterpreterError::ExpectedAnExpression(cause)) => println!("{}", cause),
            Err(InterpreterError::VerbNotFound(verb_name)) => {
                println!("Verb not found: {}", verb_name)
            }
            Err(InterpreterError::Unimplemented(part_name)) => {
                println!("Part or form is not implemented: {}", part_name)
            }
            Err(InterpreterError::ReadingFailed(err)) => match err {
                ReaderError::EoF => println!("Unexpected end of file"),
                ReaderError::ExpectedADigit(bad_char) => {
                    println!("Expected a digit, got '{}'", bad_char)
                }
                ReaderError::UnexpectedCharacter(bad_char) => {
                    println!("Unexpected character: '{}'", bad_char)
                }
            },
        }
    }
}
