#[derive(Debug, PartialEq)]
pub enum Object {
    Primitive(PrimitiveValue),
}

impl Clone for Object {
    fn clone(&self) -> Self {
        match self {
            Self::Primitive(value) => Self::Primitive(value.clone()),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::PrimitiveValue;

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
}
