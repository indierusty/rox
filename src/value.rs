use std::ops::{Add, Div, Mul, Not, Sub};

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Value {
    Bool(bool),
    Num(f64),
    Nil,
    String(String),
}

impl Not for Value {
    type Output = Result<Value, String>;

    fn not(self) -> Self::Output {
        match self {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            _ => Err("oprands must be boolean".to_string()),
        }
    }
}

impl Div for Value {
    type Output = Result<Value, String>;

    fn div(self, rhs: Self) -> Self::Output {
        if let Value::Num(a) = self {
            if let Value::Num(b) = rhs {
                return Ok(Value::Num(a / b));
            }
        }

        return Err("Both operands must be of number type.".to_string());
    }
}

impl Mul for Value {
    type Output = Result<Value, String>;

    fn mul(self, rhs: Self) -> Self::Output {
        if let Value::Num(a) = self {
            if let Value::Num(b) = rhs {
                return Ok(Value::Num(a * b));
            }
        }

        return Err("Both operands must be of number type.".to_string());
    }
}

impl Sub for Value {
    type Output = Result<Value, String>;

    fn sub(self, rhs: Self) -> Self::Output {
        if let Value::Num(a) = self {
            if let Value::Num(b) = rhs {
                return Ok(Value::Num(a - b));
            }
        }

        return Err("Both operands must be of number type.".to_string());
    }
}

impl Add for Value {
    type Output = Result<Value, String>;

    fn add(self, rhs: Self) -> Self::Output {
        if let Value::Num(a) = self {
            if let Value::Num(b) = rhs {
                return Ok(Value::Num(a + b));
            }
        }

        if let Value::String(mut a) = self {
            if let Value::String(b) = rhs {
                a.push_str(&b);
                return Ok(Value::String(a));
            }
        }

        return Err("both oprands must be number or string type.".to_string());
    }
}
