use crate::object::Object;

pub fn add(args: Vec<Object>) -> Object {
    if let Some(a) = args[0].get_integer() {
        if let Some(b) = args[1].get_integer() {
            return Object::Integer(a + b);
        } else if let Some(b) = args[1].get_float() {
            return Object::Float(a as f64 + b);
        }
    } else if let Some(a) = args[0].get_float() {
        if let Some(b) = args[1].get_integer() {
            return Object::Float(a + b as f64);
        } else if let Some(b) = args[1].get_float() {
            return Object::Float(a + b);
        }
    }

    Object::Noop
}

pub fn sub(args: Vec<Object>) -> Object {
    if let Some(a) = args[0].get_integer() {
        if let Some(b) = args[1].get_integer() {
            return Object::Integer(a - b);
        } else if let Some(b) = args[1].get_float() {
            return Object::Float(a as f64 - b);
        }
    } else if let Some(a) = args[0].get_float() {
        if let Some(b) = args[1].get_integer() {
            return Object::Float(a - b as f64);
        } else if let Some(b) = args[1].get_float() {
            return Object::Float(a - b);
        }
    }

    Object::Noop
}

pub fn mul(args: Vec<Object>) -> Object {
    if let Some(a) = args[0].get_integer() {
        if let Some(b) = args[1].get_integer() {
            return Object::Integer(a * b);
        } else if let Some(b) = args[1].get_float() {
            return Object::Float(a as f64 * b);
        }
    } else if let Some(a) = args[0].get_float() {
        if let Some(b) = args[1].get_integer() {
            return Object::Float(a * b as f64);
        } else if let Some(b) = args[1].get_float() {
            return Object::Float(a * b);
        }
    }

    Object::Noop
}

pub fn div(args: Vec<Object>) -> Object {
    if let Some(a) = args[0].get_integer() {
        if let Some(b) = args[1].get_integer() {
            if a % b == 0 {
                return Object::Integer(a / b);
            } else {
                return Object::Float(a as f64 / b as f64);
            }
        } else if let Some(b) = args[1].get_float() {
            return Object::Float(a as f64 / b);
        }
    } else if let Some(a) = args[0].get_float() {
        if let Some(b) = args[1].get_integer() {
            return Object::Float(a / b as f64);
        } else if let Some(b) = args[1].get_float() {
            return Object::Float(a / b);
        }
    }

    Object::Noop
}
