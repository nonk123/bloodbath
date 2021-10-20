use crate::object::FunctionImplementation;
use crate::object::Object;
use crate::reader::Reader;
use crate::reader::ReaderError;
use crate::reader::Token;
use std::collections::HashMap;
use std::rc::Rc;

pub enum Expression {
    Constant(Object),
    Variable(String),
    Compound(Vec<Expression>),
    Set(String, Box<Expression>),
    FunctionCall(FunctionImplementation, Vec<Expression>),
    If(Box<Expression>, Box<Expression>, Option<Box<Expression>>),
}

impl Expression {
    pub fn evaluate(&self, interpreter: &mut Bloodbath) -> Object {
        match self {
            Self::Constant(result) => result.clone(),
            Self::Variable(name) => interpreter.variable_get(&name),
            Self::Compound(expressions) => {
                let mut result = Object::Noop;

                for expression in expressions {
                    result = expression.evaluate(interpreter);
                }

                result
            }
            Self::Set(name, value) => {
                let value = value.evaluate(interpreter);
                interpreter.variable_set(&name, value.clone());
                value
            }
            Self::FunctionCall(implementation, args) => {
                let args = args.iter().map(|x| x.evaluate(interpreter)).collect();
                implementation.call(args)
            }
            Self::If(condition, if_true, otherwise) => match condition.evaluate(interpreter) {
                Object::Noop => match otherwise {
                    Some(otherwise) => otherwise.evaluate(interpreter),
                    None => Object::Noop,
                },
                _ => if_true.evaluate(interpreter),
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParserError {
    ReadingFailed(ReaderError),
    ExpectedAnExpression(String),
    ExpectedAnIdentifier(String),
    UnterminatedCompoundExpression,
    UnexpectedBrace,
}

pub struct Bloodbath {
    environment: HashMap<String, Object>,
}

type ParserResult = Result<Expression, ParserError>;

impl Bloodbath {
    pub fn new() -> Self {
        let mut us = Self {
            environment: HashMap::new(),
        };

        us.register(&"+".into(), 2, crate::builtins::add);
        us.register(&"-".into(), 2, crate::builtins::sub);
        us.register(&"*".into(), 2, crate::builtins::mul);
        us.register(&"/".into(), 2, crate::builtins::div);

        us
    }

    pub fn variable_get(&mut self, variable_name: &String) -> Object {
        match self.environment.get(variable_name) {
            Some(value) => value.clone(),
            None => {
                self.variable_set(variable_name, Object::Noop);
                Object::Noop
            }
        }
    }

    pub fn variable_set(&mut self, variable_name: &String, new_value: Object) {
        self.environment
            .insert(variable_name.clone(), new_value.clone());
    }

    pub fn register<T>(&mut self, function_name: &String, argument_count: u16, builtin: T)
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

    fn expect_keyword(
        &mut self,
        tokens: &mut Vec<Token>,
        expected_name: &str,
    ) -> Result<(), ParserError> {
        let err = ParserError::ExpectedAnIdentifier(format!("Keyword `{}`", expected_name));

        if tokens.is_empty() {
            return Err(err);
        }

        match tokens.remove(0) {
            Token::Identifier(name) => {
                if name == expected_name.to_string() {
                    Ok(())
                } else {
                    Err(err)
                }
            }
            _ => Err(err),
        }
    }

    fn check_keyword(&mut self, tokens: &mut Vec<Token>, expected_name: &str) -> bool {
        !tokens.is_empty()
            && match &tokens[0] {
                Token::Identifier(name) => {
                    if *name == expected_name.to_string() {
                        tokens.remove(0);
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            }
    }

    fn parse_variable(&mut self, name: &String, tokens: &mut Vec<Token>) -> ParserResult {
        let variable_value = self.variable_get(&name);

        match variable_value {
            Object::Function {
                argument_count,
                implementation,
            } => {
                let mut arguments = Vec::new();

                for count in 0..argument_count {
                    if tokens.is_empty() {
                        return Err(ParserError::ExpectedAnExpression(format!(
                            "Expected {} arguments after `{}`, got {}",
                            argument_count, name, count
                        )));
                    }

                    arguments.push(self.parse_expression(tokens)?);
                }

                Ok(Expression::FunctionCall(implementation, arguments))
            }
            _ => Ok(Expression::Variable(name.clone())),
        }
    }

    fn parse_compound(&mut self, tokens: &mut Vec<Token>) -> ParserResult {
        if tokens.is_empty() {
            return Err(ParserError::UnterminatedCompoundExpression);
        }

        let mut expressions = Vec::new();

        loop {
            if tokens[0] == Token::RightBrace {
                tokens.remove(0);
                return Ok(Expression::Compound(expressions));
            }

            expressions.push(self.parse_expression(tokens)?);

            if tokens.is_empty() {
                return Err(ParserError::UnterminatedCompoundExpression);
            }
        }
    }

    fn parse_identity(&mut self, tokens: &mut Vec<Token>) -> ParserResult {
        if tokens.is_empty() {
            return Err(ParserError::ExpectedAnExpression(
                "`identity` must be followed by a constant or a variable name".into(),
            ));
        }

        return match tokens.remove(0) {
            Token::Identifier(name) => {
                if name == "noop" {
                    Ok(Expression::Constant(Object::Noop))
                } else {
                    Ok(Expression::Variable(name))
                }
            }
            Token::IntegerConstant(value) => Ok(Expression::Constant(Object::Integer(value))),
            Token::FloatConstant(value) => Ok(Expression::Constant(Object::Float(value))),
            Token::LeftBrace | Token::RightBrace => Err(ParserError::UnexpectedBrace),
        };
    }

    fn parse_set(&mut self, tokens: &mut Vec<Token>) -> ParserResult {
        let usage =
            "`set` must be followed by a variable name and the variable's new value".to_string();

        if tokens.is_empty() {
            return Err(ParserError::ExpectedAnIdentifier(usage));
        }

        let variable_name = match tokens.remove(0) {
            Token::Identifier(name) => name,
            _ => return Err(ParserError::ExpectedAnIdentifier(usage)),
        };

        if tokens.is_empty() {
            return Err(ParserError::ExpectedAnExpression(usage));
        }

        let new_value = self.parse_expression(tokens)?;

        Ok(Expression::Set(variable_name, Box::new(new_value)))
    }

    fn parse_if(&mut self, tokens: &mut Vec<Token>) -> ParserResult {
        if tokens.is_empty() {
            return Err(ParserError::ExpectedAnExpression(
                "`if` must be followed by a condition".into(),
            ));
        }

        let condition = Box::new(self.parse_expression(tokens)?);

        self.expect_keyword(tokens, "then")?;

        if tokens.is_empty() {
            return Err(ParserError::ExpectedAnExpression(
                "`then` must be followed by an expression".into(),
            ));
        }

        let if_true = Box::new(self.parse_expression(tokens)?);

        let otherwise = if self.check_keyword(tokens, "else") {
            if tokens.is_empty() {
                return Err(ParserError::ExpectedAnExpression(
                    "`else` must be followed by an expression".into(),
                ));
            }

            Some(Box::new(self.parse_expression(tokens)?))
        } else {
            None
        };

        Ok(Expression::If(condition, if_true, otherwise))
    }

    fn parse_expression(&mut self, tokens: &mut Vec<Token>) -> ParserResult {
        match tokens.remove(0) {
            Token::Identifier(name) => match name.as_str() {
                "noop" => Ok(Expression::Constant(Object::Noop)),
                "identity" => self.parse_identity(tokens),
                "set" => self.parse_set(tokens),
                "if" => self.parse_if(tokens),
                _ => self.parse_variable(&name, tokens),
            },
            Token::IntegerConstant(value) => Ok(Expression::Constant(Object::Integer(value))),
            Token::FloatConstant(value) => Ok(Expression::Constant(Object::Float(value))),
            Token::LeftBrace => self.parse_compound(tokens),
            Token::RightBrace => Err(ParserError::UnexpectedBrace),
        }
    }

    #[cfg(test)]
    pub fn eval_str(&mut self, input: &str) -> Result<Object, ParserError> {
        self.eval(input.into())
    }

    pub fn eval(&mut self, input: String) -> Result<Object, ParserError> {
        let mut reader = Reader::new(input);

        let mut tokens = reader
            .tokenise()
            .or_else(|err| Err(ParserError::ReadingFailed(err)))?;

        let mut result = Object::Noop;

        while !tokens.is_empty() {
            result = self.parse_expression(&mut tokens)?.evaluate(self);
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

        assert_eq!(
            bloodbath.eval_str("if 0 then 42 else 0"),
            Ok(Object::Integer(42))
        );

        assert_eq!(
            bloodbath.eval_str("if noop then 42 else 0"),
            Ok(Object::Integer(0))
        );

        assert_eq!(bloodbath.eval_str("if noop then 42"), Ok(Object::Noop));

        assert_eq!(
            bloodbath.eval_str("if noop then 1 else if noop then 2 else 3"),
            Ok(Object::Integer(3))
        );
    }

    #[test]
    fn test_variables() {
        let mut bloodbath = Bloodbath::new();

        assert_eq!(bloodbath.eval_str("set a 10"), Ok(Object::Integer(10)));
        assert_eq!(bloodbath.eval_str("set b 20"), Ok(Object::Integer(20)));
        assert_eq!(bloodbath.eval_str("set c + a b"), Ok(Object::Integer(30)));
        assert_eq!(bloodbath.eval_str("set + c"), Ok(Object::Integer(30)));
        assert_eq!(bloodbath.eval_str("+"), Ok(Object::Integer(30)));
        assert_eq!(bloodbath.eval_str("identity +"), Ok(Object::Integer(30)));
    }

    #[test]
    fn test_compound() {
        let mut bloodbath = Bloodbath::new();

        assert_eq!(bloodbath.eval_str("{1 2 3}"), Ok(Object::Integer(3)));
        assert_eq!(bloodbath.eval_str("{set a 5}"), Ok(Object::Integer(5)));
        assert_eq!(bloodbath.eval_str("{set b + 5 a}"), Ok(Object::Integer(10)));

        assert_eq!(
            bloodbath.eval_str("if {a noop} then 1 else 0"),
            Ok(Object::Integer(0))
        );
    }
}
