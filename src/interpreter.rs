use crate::object::FunctionImplementation;
use crate::object::Object;
use crate::reader::Reader;
use crate::reader::ReaderError;
use crate::reader::Token;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
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
        let mut us = Self {
            environment: HashMap::new(),
        };

        us.register("+".into(), 2, crate::builtins::add);
        us.register("-".into(), 2, crate::builtins::sub);
        us.register("*".into(), 2, crate::builtins::mul);
        us.register("/".into(), 2, crate::builtins::div);

        us
    }

    pub fn variable_get(&mut self, variable_name: &String) -> Object {
        match self.environment.get(variable_name) {
            Some(value) => value.clone(),
            None => {
                self.environment.insert(variable_name.clone(), Object::Noop);
                Object::Noop
            }
        }
    }

    pub fn variable_set(&mut self, variable_name: String, new_value: Object) {
        self.environment.insert(variable_name, new_value.clone());
    }

    pub fn register<T>(&mut self, function_name: String, argument_count: u16, builtin: T)
    where
        T: Fn(Vec<Object>) -> Object + 'static,
    {
        self.variable_set(
            function_name,
            Object::Function {
                argument_count,
                implementation: FunctionImplementation::Builtin(Rc::new(builtin)),
            },
        );
    }

    fn eval_variable(&mut self, name: &String, tokens: &mut Vec<Token>) -> InterpreterResult {
        let variable_value = self.variable_get(&name);

        match variable_value {
            Object::Function {
                argument_count,
                implementation,
            } => {
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
                    Ok(Object::Noop)
                } else if ["identity", "set"].contains(&name.as_str()) {
                    Err(InterpreterError::IllegalIdentity(format!(
                        "Cannot use `identity` on syntax form `{}`",
                        name
                    )))
                } else {
                    Ok(self.variable_get(&name))
                }
            }
            Token::IntegerConstant(value) => Ok(Object::Integer(value)),
            Token::FloatConstant(value) => Ok(Object::Float(value)),
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
                    return Ok(Object::Noop);
                } else if name == "identity" {
                    self.eval_identity(tokens)
                } else if name == "set" {
                    self.eval_set(tokens)
                } else {
                    self.eval_variable(&name, tokens)
                }
            }
            Token::IntegerConstant(value) => Ok(Object::Integer(value)),
            Token::FloatConstant(value) => Ok(Object::Float(value)),
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

        let mut result = Object::Noop;

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

        assert_eq!(bloodbath.eval_str("noop"), Ok(Object::Noop),);
        assert_eq!(bloodbath.eval_str("identity 1"), Ok(Object::Integer(1)));
        assert_eq!(bloodbath.eval_str("+ 1 2"), Ok(Object::Integer(3)));
        assert_eq!(bloodbath.eval_str("+ 1 + 1 1"), Ok(Object::Integer(3)));
        assert_eq!(bloodbath.eval_str("+ + 1 1 1"), Ok(Object::Integer(3)));
    }

    #[test]
    fn test_variables() {
        let mut bloodbath = Bloodbath::new();

        assert_eq!(bloodbath.eval_str("set a 10"), Ok(Object::Integer(10)));
        assert_eq!(bloodbath.eval_str("set b 20"), Ok(Object::Integer(20)));
        assert_eq!(bloodbath.eval_str("set c + a b"), Ok(Object::Integer(30)));
        assert_eq!(bloodbath.eval_str("set + c"), Ok(Object::Integer(30)));
        assert_eq!(bloodbath.eval_str("+"), Ok(Object::Integer(30)));
    }
}
