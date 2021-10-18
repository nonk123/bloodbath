use crate::object::FunctionImplementation;
use crate::object::Object;
use crate::object::PrimitiveValue;
use crate::object::ReferenceValue;
use crate::reader::Reader;
use crate::reader::ReaderError;
use crate::reader::Token;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub enum InterpreterError {
    ReadingFailed(ReaderError),
    ExpectedAnExpression(String),
    ExpectedAnIdentifier(String),
    IllegalIdentity(String),
}

pub struct Bloodbath {
    environment: HashMap<String, Object>,
}

type InterpreterResult = Result<Object, InterpreterError>;

impl Bloodbath {
    pub fn new() -> Self {
        let mut environment = HashMap::new();

        let make_builtin = |argument_count, builtin| {
            Object::Reference(ReferenceValue::Function {
                argument_count,
                implementation: FunctionImplementation::Builtin(Rc::new(builtin)),
            })
        };

        environment.insert(
            "+".into(),
            make_builtin(2, |args: Vec<Object>| {
                let noop = Object::Primitive(PrimitiveValue::Noop);

                match args[0] {
                    Object::Primitive(PrimitiveValue::Integer(x)) => match args[1] {
                        Object::Primitive(PrimitiveValue::Integer(y)) => {
                            Object::Primitive(PrimitiveValue::Integer(x + y))
                        }
                        Object::Primitive(PrimitiveValue::Float(y)) => {
                            Object::Primitive(PrimitiveValue::Float(x as f64 + y))
                        }
                        _ => noop,
                    },
                    Object::Primitive(PrimitiveValue::Float(x)) => match args[1] {
                        Object::Primitive(PrimitiveValue::Integer(y)) => {
                            Object::Primitive(PrimitiveValue::Float(x + y as f64))
                        }
                        Object::Primitive(PrimitiveValue::Float(y)) => {
                            Object::Primitive(PrimitiveValue::Float(x + y))
                        }
                        _ => noop,
                    },
                    _ => noop,
                }
            }),
        );

        Self { environment }
    }

    pub fn variable_get(&mut self, variable_name: &String) -> Object {
        match self.environment.get(variable_name) {
            Some(value) => value.clone(),
            None => {
                let noop = Object::Primitive(PrimitiveValue::Noop);
                self.environment.insert(variable_name.clone(), noop.clone());
                noop
            }
        }
    }

    pub fn variable_set(&mut self, variable_name: String, new_value: Object) {
        self.environment.insert(variable_name, new_value.clone());
    }

    fn eval_variable(&mut self, name: &String, tokens: &mut Vec<Token>) -> InterpreterResult {
        let variable_value = self.variable_get(&name);

        match variable_value {
            Object::Reference(ReferenceValue::Function {
                argument_count,
                implementation,
            }) => {
                let mut arguments = Vec::new();

                for count in 0..argument_count {
                    if tokens.is_empty() {
                        return Err(InterpreterError::ExpectedAnExpression(format!(
                            "Expected {} arguments after `{}`, got {}",
                            argument_count, name, count
                        )));
                    }

                    arguments.push(self.eval_expression(tokens)?);
                }

                Ok(implementation.call(arguments))
            }
            _ => Ok(variable_value.clone()),
        }
    }

    fn eval_identity(&mut self, tokens: &mut Vec<Token>) -> InterpreterResult {
        if tokens.is_empty() {
            return Err(InterpreterError::ExpectedAnExpression(
                "`identity` must be followed by a constant or a variable name".into(),
            ));
        }

        return match tokens.remove(0) {
            Token::Identifier(name) => {
                if name == "noop" {
                    Ok(Object::Primitive(PrimitiveValue::Noop))
                } else if ["identity", "set"].contains(&name.as_str()) {
                    Err(InterpreterError::IllegalIdentity(format!(
                        "Cannot use `identity` on syntax form `{}`",
                        name
                    )))
                } else {
                    Ok(self.variable_get(&name))
                }
            }
            Token::IntegerConstant(value) => Ok(Object::Primitive(PrimitiveValue::Integer(value))),
            Token::FloatConstant(value) => Ok(Object::Primitive(PrimitiveValue::Float(value))),
        };
    }

    fn eval_set(&mut self, tokens: &mut Vec<Token>) -> InterpreterResult {
        let usage =
            "`set` must be followed by a variable name and the variable's new value".to_string();

        let variable_name = match tokens.remove(0) {
            Token::Identifier(name) => name,
            _ => return Err(InterpreterError::ExpectedAnIdentifier(usage)),
        };

        if tokens.is_empty() {
            return Err(InterpreterError::ExpectedAnIdentifier(usage));
        }

        let new_value = self.eval_expression(tokens)?;

        self.variable_set(variable_name, new_value.clone());

        Ok(new_value)
    }

    fn eval_expression(&mut self, tokens: &mut Vec<Token>) -> InterpreterResult {
        match tokens.remove(0) {
            Token::Identifier(name) => {
                if name == "noop" {
                    return Ok(Object::Primitive(PrimitiveValue::Noop));
                } else if name == "identity" {
                    self.eval_identity(tokens)
                } else if name == "set" {
                    self.eval_set(tokens)
                } else {
                    self.eval_variable(&name, tokens)
                }
            }
            Token::IntegerConstant(value) => Ok(Object::Primitive(PrimitiveValue::Integer(value))),
            Token::FloatConstant(value) => Ok(Object::Primitive(PrimitiveValue::Float(value))),
        }
    }

    #[cfg(test)]
    pub fn eval_str(&mut self, input: &str) -> InterpreterResult {
        self.eval(input.into())
    }

    pub fn eval(&mut self, input: String) -> InterpreterResult {
        let mut reader = Reader::new(input);

        let mut tokens = reader
            .tokenise()
            .or_else(|err| Err(InterpreterError::ReadingFailed(err)))?;

        let mut result = Object::Primitive(PrimitiveValue::Noop);

        while !tokens.is_empty() {
            result = self.eval_expression(&mut tokens)?;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval() {
        let mut bloodbath = Bloodbath::new();

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

    #[test]
    fn test_variables() {
        let mut bloodbath = Bloodbath::new();

        assert_eq!(
            bloodbath.eval_str("set a 10").unwrap(),
            Object::Primitive(PrimitiveValue::Integer(10))
        );

        assert_eq!(
            bloodbath.eval_str("set b 20").unwrap(),
            Object::Primitive(PrimitiveValue::Integer(20))
        );

        assert_eq!(
            bloodbath.eval_str("set c + a b").unwrap(),
            Object::Primitive(PrimitiveValue::Integer(30))
        );

        assert_eq!(
            bloodbath.eval_str("set + c").unwrap(),
            Object::Primitive(PrimitiveValue::Integer(30))
        );

        assert_eq!(
            bloodbath.eval_str("+").unwrap(),
            Object::Primitive(PrimitiveValue::Integer(30))
        );
    }
}
