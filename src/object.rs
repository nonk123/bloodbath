use std::fmt::Debug;
use std::fmt::Formatter;
use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Primitive(PrimitiveValue),
    Reference(ReferenceValue),
}

#[derive(Debug, PartialEq)]
pub enum PrimitiveValue {
    Noop,
    Integer(i64),
    Float(f64),
}

impl Clone for PrimitiveValue {
    fn clone(&self) -> Self {
        match self {
            Self::Noop => Self::Noop,
            Self::Integer(value) => Self::Integer(*value),
            Self::Float(value) => Self::Float(*value),
        }
    }
}

impl PrimitiveValue {
    #[cfg(test)]
    pub fn is_noop(&self) -> bool {
        match self {
            Self::Noop => true,
            _ => false,
        }
    }

    #[cfg(test)]
    pub fn get_integer(&self) -> Option<i64> {
        match self {
            Self::Integer(value) => Some(*value),
            _ => None,
        }
    }

    #[cfg(test)]
    pub fn get_float(&self) -> Option<f64> {
        match self {
            Self::Float(value) => Some(*value),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ReferenceValue {
    Function {
        argument_count: u16,
        implementation: FunctionImplementation,
    },
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
    fn test_noop() {
        assert!(PrimitiveValue::Noop.is_noop());
    }

    #[test]
    fn test_integer() {
        let a = PrimitiveValue::Integer(1);
        let b = PrimitiveValue::Integer(2);

        let a_value = a.get_integer().unwrap();
        let b_value = b.get_integer().unwrap();

        assert_eq!(a_value + b_value, 3);
    }

    #[test]
    fn test_float() {
        let a = PrimitiveValue::Float(1.0);
        let b = PrimitiveValue::Float(2.0);

        let a_value = a.get_float().unwrap();
        let b_value = b.get_float().unwrap();

        assert!((a_value + b_value - 3.0).abs() <= 1e-3);
    }

    #[test]
    fn test_builtin_function() {
        let function = ReferenceValue::Function {
            argument_count: 1,
            implementation: FunctionImplementation::Builtin(Rc::new(|args| {
                assert_eq!(args.len(), 1);

                match args[0] {
                    Object::Primitive(PrimitiveValue::Integer(x)) => {
                        Object::Primitive(PrimitiveValue::Integer(x + 1))
                    }
                    _ => unreachable!(),
                }
            })),
        };

        match function {
            ReferenceValue::Function {
                argument_count,
                implementation,
            } => {
                let sixty_eight = Object::Primitive(PrimitiveValue::Integer(68));
                let sixty_nine = Object::Primitive(PrimitiveValue::Integer(69));

                assert_eq!(argument_count, 1);
                assert_eq!(implementation.call(vec![sixty_eight]), sixty_nine);
            }
        }
    }
}
