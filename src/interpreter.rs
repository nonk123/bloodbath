use crate::object::Object;
use crate::object::PrimitiveValue;

struct Function {
    argument_count: u16,
    action: Box<dyn Fn(Vec<Object>) -> Object>,
}

impl Function {
    fn new<F>(argument_count: u16, action: F) -> Self
    where
        F: Fn(Vec<Object>) -> Object + 'static,
    {
        Self {
            argument_count,
            action: Box::new(action),
        }
    }

    fn call(&self, arguments: Vec<Object>) -> Object {
        (self.action)(arguments)
    }
}

#[derive(Debug, PartialEq)]
enum Token {
    Identifier(String),
    IntegerConstant(i64),
    FloatConstant(f64),
}

#[derive(Debug)]
pub enum ReaderError {
    EoF,
    ExpectedADigit(char),
    UnexpectedCharacter(char),
}

struct Reader {
    input: String,
    position: usize,
}

impl Reader {
    fn new(input: String) -> Self {
        Self { input, position: 0 }
    }

    fn peek(&self, amount: usize) -> Result<char, ReaderError> {
        let position = self.position + amount;
        self.input.chars().nth(position).ok_or(ReaderError::EoF)
    }

    fn current(&self) -> Result<char, ReaderError> {
        self.peek(0)
    }

    fn next(&mut self) -> Result<char, ReaderError> {
        self.position += 1;
        self.current()
    }

    fn next_or_eof(&mut self) -> Result<bool, ReaderError> {
        match self.next() {
            Err(ReaderError::EoF) => Ok(true),
            Err(err) => Err(err),
            Ok(_) => Ok(false),
        }
    }

    fn is_eof(&self) -> bool {
        self.position >= self.input.len()
    }

    fn is_separator(&self, input: &char) -> bool {
        [' ', '\t', '\n', '\t'].contains(input)
    }

    fn read_number(&mut self) -> Result<Token, ReaderError> {
        let mut whole = 0;

        let mut fractional: Option<i64> = None;
        let mut fractional_power = 1;

        while !self.is_separator(&self.current()?) && self.current()? != '.' {
            let digit = self.current()? as i64 - '0' as i64;

            if digit < 0 || digit > 9 {
                return Err(ReaderError::ExpectedADigit(self.current()?));
            }

            whole *= 10;
            whole += digit;

            if self.next_or_eof()? {
                return Ok(Token::IntegerConstant(whole));
            }
        }

        if self.current()? == '.' {
            self.next()?;

            fractional = Some(0);

            while !self.is_separator(&self.current()?) {
                let digit = self.current()? as i64 - '0' as i64;

                if digit < 0 || digit > 9 {
                    return Err(ReaderError::ExpectedADigit(self.current()?));
                }

                fractional = Some(fractional.unwrap() * 10 + digit);
                fractional_power *= 10;

                if self.next_or_eof()? {
                    break;
                }
            }
        }

        if let Some(fractional) = fractional {
            Ok(Token::FloatConstant(
                whole as f64 + fractional as f64 / fractional_power as f64,
            ))
        } else {
            Ok(Token::IntegerConstant(whole))
        }
    }

    fn read_identifier(&mut self) -> Result<Token, ReaderError> {
        let mut identifier = String::new();

        loop {
            let mut is_legal = false;

            let legal_ranges = [
                ('a'..='z'),
                ('A'..='Z'),
                ('0'..='9'),
                ('!'..='!'),
                ('#'..='/'),
                (':'..=':'),
                ('<'..='@'),
                ('\\'..='\\'),
                ('^'..='`'),
                ('|'..='|'),
                ('~'..='~'),
            ];

            for range in legal_ranges {
                if range.contains(&self.current()?) {
                    is_legal = true;
                    break;
                }
            }

            if !is_legal {
                return Err(ReaderError::UnexpectedCharacter(self.current()?));
            }

            identifier.push(self.current()?);

            if self.next_or_eof()? || self.is_separator(&self.current()?) {
                return Ok(Token::Identifier(identifier));
            }
        }
    }

    fn tokenise(&mut self) -> Result<Vec<Token>, ReaderError> {
        let mut tokens = Vec::new();

        while !self.is_eof() {
            // Skip all whitespace. Return `result` upon reaching EoF.
            while self.is_separator(&self.current()?) {
                if self.next_or_eof()? {
                    break;
                }
            }

            if ('0'..='9').contains(&self.current()?) {
                tokens.push(self.read_number()?);
            } else {
                tokens.push(self.read_identifier()?);
            }
        }

        Ok(tokens)
    }
}

#[derive(Debug)]
pub enum InterpreterError {
    ExpectedAnExpression,
    ReadingFailed(ReaderError),
    VerbNotFound(String),
    Unimplemented(String),
}

pub struct Bloodbath {}

impl Bloodbath {
    pub fn new() -> Self {
        Self {}
    }

    fn get_verb(&self, name: &String) -> Option<Function> {
        match name.as_str() {
            "+" => Some(Function::new(2, |args| match args[0] {
                Object::Primitive(PrimitiveValue::Integer(a)) => match args[1] {
                    // int + int = int
                    Object::Primitive(PrimitiveValue::Integer(b)) => {
                        Object::Primitive(PrimitiveValue::Integer(a + b))
                    }
                    // int + float = float
                    Object::Primitive(PrimitiveValue::Float(b)) => {
                        Object::Primitive(PrimitiveValue::Float(a as f64 + b))
                    }
                    // int + noop = noop
                    _ => Object::Primitive(PrimitiveValue::Noop),
                },
                Object::Primitive(PrimitiveValue::Float(a)) => match args[1] {
                    // float + int = float
                    Object::Primitive(PrimitiveValue::Integer(b)) => {
                        Object::Primitive(PrimitiveValue::Float(a + b as f64))
                    }
                    // float + float = float
                    Object::Primitive(PrimitiveValue::Float(b)) => {
                        Object::Primitive(PrimitiveValue::Float(a + b))
                    }
                    // float + noop = noop
                    _ => Object::Primitive(PrimitiveValue::Noop),
                },
                // noop + any = noop
                _ => Object::Primitive(PrimitiveValue::Noop),
            })),
            _ => None,
        }
    }

    fn eval_tokens(&self, mut tokens: &mut Vec<Token>) -> Result<Object, InterpreterError> {
        match tokens.remove(0) {
            Token::Identifier(name) => {
                if name == "noop".to_string() {
                    return Ok(Object::Primitive(PrimitiveValue::Noop));
                }

                if name == "identity" {
                    if tokens.is_empty() {
                        return Err(InterpreterError::ExpectedAnExpression);
                    }

                    return match tokens.remove(0) {
                        Token::Identifier(name) => {
                            if name == "noop".to_string() {
                                Ok(Object::Primitive(PrimitiveValue::Noop))
                            } else {
                                // TODO: return a variable's value even if it refers to a function.
                                Err(InterpreterError::Unimplemented("identity".into()))
                            }
                        }
                        Token::IntegerConstant(value) => {
                            Ok(Object::Primitive(PrimitiveValue::Integer(value)))
                        }
                        Token::FloatConstant(value) => {
                            Ok(Object::Primitive(PrimitiveValue::Float(value)))
                        }
                    };
                }

                let verb = self
                    .get_verb(&name)
                    .ok_or(InterpreterError::VerbNotFound(name))?;

                let mut arguments = Vec::new();

                for _ in 0..verb.argument_count {
                    if tokens.is_empty() {
                        return Err(InterpreterError::ExpectedAnExpression);
                    }

                    arguments.push(self.eval_tokens(&mut tokens)?);
                }

                Ok(verb.call(arguments))
            }
            Token::IntegerConstant(value) => Ok(Object::Primitive(PrimitiveValue::Integer(value))),
            Token::FloatConstant(value) => Ok(Object::Primitive(PrimitiveValue::Float(value))),
        }
    }

    pub fn eval_str(&self, input: &str) -> Result<Object, InterpreterError> {
        self.eval(input.into())
    }

    pub fn eval(&self, input: String) -> Result<Object, InterpreterError> {
        let mut reader = Reader::new(input);

        let mut tokens = reader
            .tokenise()
            .or_else(|err| Err(InterpreterError::ReadingFailed(err)))?;

        let mut result = Object::Primitive(PrimitiveValue::Noop);

        while !tokens.is_empty() {
            result = self.eval_tokens(&mut tokens)?;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenise() {
        let mut reader = Reader::new("123".into());
        let result = reader.tokenise().unwrap();

        assert!(result.len() == 1);
        assert_eq!(result[0], Token::IntegerConstant(123));

        let mut reader = Reader::new("123 42.69".into());
        let result = reader.tokenise().unwrap();

        assert!(result.len() == 2);

        match result[1] {
            Token::FloatConstant(value) => assert!((value - 42.69).abs() <= 1e-3),
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_eval() {
        let bloodbath = Bloodbath::new();

        assert_eq!(
            bloodbath.eval_str("noop").unwrap(),
            Object::Primitive(PrimitiveValue::Noop),
        );

        assert_eq!(
            bloodbath.eval_str("identity 1").unwrap(),
            Object::Primitive(PrimitiveValue::Integer(1))
        );

        assert_eq!(
            bloodbath.eval_str("+ 1 2").unwrap(),
            Object::Primitive(PrimitiveValue::Integer(3)),
        );

        assert_eq!(
            bloodbath.eval_str("+ 1 + 1 1").unwrap(),
            Object::Primitive(PrimitiveValue::Integer(3)),
        );

        assert_eq!(
            bloodbath.eval_str("+ + 1 1 1").unwrap(),
            Object::Primitive(PrimitiveValue::Integer(3)),
        );
    }
}
