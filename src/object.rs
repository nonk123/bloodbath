pub enum PrimitiveValue {
    Noop,
    Integer(i64),
    Float(f64),
}

impl PrimitiveValue {
    pub fn is_noop(&self) -> bool {
        match self {
            Self::Noop => true,
            _ => false,
        }
    }

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
