use std::fmt::Debug;
use std::fmt::Formatter;
use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Noop,
    Integer(i64),
    Float(f64),
    Function {
        argument_count: u16,
        implementation: FunctionImplementation,
    },
}

impl Object {
    pub fn get_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(value) => Some(*value),
            _ => None,
        }
    }

    pub fn get_float(&self) -> Option<f64> {
        match self {
            Self::Float(value) => Some(*value),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub enum FunctionImplementation {
    Builtin(Rc<dyn Fn(Vec<Object>) -> Object>),
}

impl Debug for FunctionImplementation {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::Builtin(_) => write!(formatter, "<builtin>")?,
        };

        Ok(())
    }
}

impl PartialEq for FunctionImplementation {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Builtin(our_impl) => match other {
                Self::Builtin(their_impl) => Rc::ptr_eq(our_impl, their_impl),
            },
        }
    }
}

impl FunctionImplementation {
    pub fn call(&self, arguments: Vec<Object>) -> Object {
        match self {
            FunctionImplementation::Builtin(action) => (action)(arguments),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer() {
        let a = Object::Integer(1);
        let b = Object::Integer(2);

        let a_value = a.get_integer().unwrap();
        let b_value = b.get_integer().unwrap();

        assert_eq!(a_value + b_value, 3);
    }

    #[test]
    fn test_float() {
        let a = Object::Float(1.0);
        let b = Object::Float(2.0);

        let a_value = a.get_float().unwrap();
        let b_value = b.get_float().unwrap();

        assert!((a_value + b_value - 3.0).abs() <= 1e-3);
    }

    #[test]
    fn test_builtin_function() {
        let function = Object::Function {
            argument_count: 1,
            implementation: FunctionImplementation::Builtin(Rc::new(|args| {
                assert_eq!(args.len(), 1);

                match args[0] {
                    Object::Integer(x) => Object::Integer(x + 1),
                    _ => unreachable!(),
                }
            })),
        };

        match function {
            Object::Function {
                argument_count,
                implementation,
            } => {
                let sixty_eight = Object::Integer(68);
                let sixty_nine = Object::Integer(69);

                assert_eq!(argument_count, 1);
                assert_eq!(implementation.call(vec![sixty_eight]), sixty_nine);
            }
            _ => unreachable!(),
        }
    }
}
