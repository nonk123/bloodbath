use crate::object::Object;
use crate::object::PrimitiveValue;
use crate::reader::Reader;
use crate::reader::ReaderError;
use crate::reader::Token;

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

#[derive(Debug)]
pub enum InterpreterError {
    ReadingFailed(ReaderError),
    ExpectedAnExpression(String),
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
                        return Err(InterpreterError::ExpectedAnExpression(
                            "`identity` must be followed by a constant or a variable name".into(),
                        ));
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
                    .ok_or(InterpreterError::VerbNotFound(name.clone()))?;

                let mut arguments = Vec::new();

                for count in 0..verb.argument_count {
                    if tokens.is_empty() {
                        return Err(InterpreterError::ExpectedAnExpression(format!(
                            "Expected {} arguments after `{}`, got {}",
                            verb.argument_count,
                            name.clone(),
                            count
                        )));
                    }

                    arguments.push(self.eval_tokens(&mut tokens)?);
                }

                Ok(verb.call(arguments))
            }
            Token::IntegerConstant(value) => Ok(Object::Primitive(PrimitiveValue::Integer(value))),
            Token::FloatConstant(value) => Ok(Object::Primitive(PrimitiveValue::Float(value))),
        }
    }

    #[cfg(test)]
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
