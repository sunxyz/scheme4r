use crate::reader::{datum::fmt_inexact_number, Datum, Reader};

use super::*;

pub(super) fn install(env: &mut Environment) {
    define_builtin(env, "number?", is_number);
    define_builtin(env, "complex?", is_complex);
    define_builtin(env, "real?", is_real);
    define_builtin(env, "rational?", is_rational);
    define_builtin(env, "integer?", is_integer);
    define_builtin(env, "exact?", is_exact);
    define_builtin(env, "inexact?", is_inexact);
    define_builtin(env, "exact-integer?", is_exact_integer);
    define_builtin(env, "finite?", is_finite);
    define_builtin(env, "infinite?", is_infinite);
    define_builtin(env, "nan?", is_nan);
    define_builtin(env, "+", add);
    define_builtin(env, "-", subtract);
    define_builtin(env, "*", multiply);
    define_builtin(env, "/", divide);
    define_builtin(env, "abs", abs);
    define_builtin(env, "zero?", is_zero);
    define_builtin(env, "positive?", is_positive);
    define_builtin(env, "negative?", is_negative);
    define_builtin(env, "odd?", is_odd);
    define_builtin(env, "even?", is_even);
    define_builtin(env, "max", max);
    define_builtin(env, "min", min);
    define_builtin(env, "quotient", quotient);
    define_builtin(env, "remainder", remainder);
    define_builtin(env, "modulo", modulo);
    define_builtin(env, "gcd", gcd);
    define_builtin(env, "lcm", lcm);
    define_builtin(env, "=", numeric_eq);
    define_builtin(env, "<", numeric_lt);
    define_builtin(env, ">", numeric_gt);
    define_builtin(env, "<=", numeric_lte);
    define_builtin(env, ">=", numeric_gte);
    define_builtin(env, "number->string", number_to_string);
    define_builtin(env, "string->number", string_to_number);
}

#[derive(Clone, Copy, Debug)]
enum Numeric {
    Integer(i64),
    Float(f64),
}

impl Numeric {
    fn from_value(name: &str, value: &Value) -> Result<Self, SchemeError> {
        match value {
            Value::Number(number) => Ok(Self::Integer(*number)),
            Value::Float(number) => Ok(Self::Float(*number)),
            other => Err(SchemeError::type_error(format!(
                "'{name}' expected a number, got {other}"
            ))),
        }
    }

    fn to_value(self) -> Value {
        match self {
            Self::Integer(value) => Value::Number(value),
            Self::Float(value) => Value::Float(value),
        }
    }

    fn to_f64(self) -> f64 {
        match self {
            Self::Integer(value) => value as f64,
            Self::Float(value) => value,
        }
    }

    fn is_zero(self) -> bool {
        match self {
            Self::Integer(value) => value == 0,
            Self::Float(value) => value == 0.0,
        }
    }

    fn is_integer(self) -> bool {
        match self {
            Self::Integer(_) => true,
            Self::Float(value) => value.is_finite() && value.fract() == 0.0,
        }
    }

    fn is_exact(self) -> bool {
        matches!(self, Self::Integer(_))
    }

    fn add(self, other: Self, name: &str) -> Result<Self, SchemeError> {
        match (self, other) {
            (Self::Integer(left), Self::Integer(right)) => left
                .checked_add(right)
                .map(Self::Integer)
                .ok_or_else(|| integer_overflow(name)),
            _ => Ok(Self::Float(self.to_f64() + other.to_f64())),
        }
    }

    fn subtract(self, other: Self, name: &str) -> Result<Self, SchemeError> {
        match (self, other) {
            (Self::Integer(left), Self::Integer(right)) => left
                .checked_sub(right)
                .map(Self::Integer)
                .ok_or_else(|| integer_overflow(name)),
            _ => Ok(Self::Float(self.to_f64() - other.to_f64())),
        }
    }

    fn multiply(self, other: Self, name: &str) -> Result<Self, SchemeError> {
        match (self, other) {
            (Self::Integer(left), Self::Integer(right)) => left
                .checked_mul(right)
                .map(Self::Integer)
                .ok_or_else(|| integer_overflow(name)),
            _ => Ok(Self::Float(self.to_f64() * other.to_f64())),
        }
    }

    fn negate(self, name: &str) -> Result<Self, SchemeError> {
        match self {
            Self::Integer(value) => value
                .checked_neg()
                .map(Self::Integer)
                .ok_or_else(|| integer_overflow(name)),
            Self::Float(value) => Ok(Self::Float(-value)),
        }
    }

    fn reciprocal(self) -> Result<Self, SchemeError> {
        if self.is_zero() {
            return Err(SchemeError::runtime("division by zero"));
        }

        match self {
            Self::Integer(1) => Ok(Self::Integer(1)),
            Self::Integer(-1) => Ok(Self::Integer(-1)),
            _ => Ok(Self::Float(1.0 / self.to_f64())),
        }
    }

    fn divide(self, other: Self) -> Result<Self, SchemeError> {
        if other.is_zero() {
            return Err(SchemeError::runtime("division by zero"));
        }

        match (self, other) {
            (Self::Integer(left), Self::Integer(right)) if left % right == 0 => {
                Ok(Self::Integer(left / right))
            }
            _ => Ok(Self::Float(self.to_f64() / other.to_f64())),
        }
    }

    fn abs(self) -> Result<Self, SchemeError> {
        match self {
            Self::Integer(value) => Ok(Self::Integer(checked_abs("abs", value)?)),
            Self::Float(value) => Ok(Self::Float(value.abs())),
        }
    }

    fn max(self, other: Self) -> Self {
        if self.to_f64() >= other.to_f64() {
            self
        } else {
            other
        }
    }

    fn min(self, other: Self) -> Self {
        if self.to_f64() <= other.to_f64() {
            self
        } else {
            other
        }
    }
}

fn is_number(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("number?", args, 1)?;
    Ok(Value::Boolean(matches!(
        args[0],
        Value::Number(_) | Value::Float(_)
    )))
}

fn is_complex(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("complex?", args, 1)?;
    Ok(Value::Boolean(matches!(
        args[0],
        Value::Number(_) | Value::Float(_)
    )))
}

fn is_real(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("real?", args, 1)?;
    Ok(Value::Boolean(matches!(
        args[0],
        Value::Number(_) | Value::Float(_)
    )))
}

fn is_rational(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("rational?", args, 1)?;
    Ok(Value::Boolean(matches!(
        args[0],
        Value::Number(_) | Value::Float(_)
    )))
}

fn is_integer(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("integer?", args, 1)?;
    Ok(Value::Boolean(
        Numeric::from_value("integer?", &args[0])?.is_integer(),
    ))
}

fn is_exact(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("exact?", args, 1)?;
    Ok(Value::Boolean(
        Numeric::from_value("exact?", &args[0])?.is_exact(),
    ))
}

fn is_inexact(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("inexact?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::Float(_))))
}

fn is_exact_integer(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("exact-integer?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::Number(_))))
}

fn is_finite(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("finite?", args, 1)?;
    Ok(Value::Boolean(match Numeric::from_value("finite?", &args[0])? {
        Numeric::Integer(_) => true,
        Numeric::Float(value) => value.is_finite(),
    }))
}

fn is_infinite(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("infinite?", args, 1)?;
    Ok(Value::Boolean(match Numeric::from_value("infinite?", &args[0])? {
        Numeric::Integer(_) => false,
        Numeric::Float(value) => value.is_infinite(),
    }))
}

fn is_nan(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("nan?", args, 1)?;
    Ok(Value::Boolean(match Numeric::from_value("nan?", &args[0])? {
        Numeric::Integer(_) => false,
        Numeric::Float(value) => value.is_nan(),
    }))
}

fn add(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    let mut sum = Numeric::Integer(0);
    for value in args {
        sum = sum.add(Numeric::from_value("+", value)?, "+")?;
    }
    Ok(sum.to_value())
}

fn subtract(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    let (first, rest) = args
        .split_first()
        .ok_or_else(|| SchemeError::arity("'-' expects at least 1 argument"))?;
    let mut total = Numeric::from_value("-", first)?;
    if rest.is_empty() {
        return Ok(total.negate("-")?.to_value());
    }
    for value in rest {
        total = total.subtract(Numeric::from_value("-", value)?, "-")?;
    }
    Ok(total.to_value())
}

fn multiply(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    let mut product = Numeric::Integer(1);
    for value in args {
        product = product.multiply(Numeric::from_value("*", value)?, "*")?;
    }
    Ok(product.to_value())
}

fn divide(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    let (first, rest) = args
        .split_first()
        .ok_or_else(|| SchemeError::arity("'/' expects at least 1 argument"))?;
    let mut total = Numeric::from_value("/", first)?;
    if rest.is_empty() {
        return Ok(total.reciprocal()?.to_value());
    }
    for value in rest {
        total = total.divide(Numeric::from_value("/", value)?)?;
    }
    Ok(total.to_value())
}

fn abs(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("abs", args, 1)?;
    Ok(Numeric::from_value("abs", &args[0])?.abs()?.to_value())
}

fn is_zero(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("zero?", args, 1)?;
    Ok(Value::Boolean(
        Numeric::from_value("zero?", &args[0])?.is_zero(),
    ))
}

fn is_positive(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("positive?", args, 1)?;
    Ok(Value::Boolean(
        Numeric::from_value("positive?", &args[0])?.to_f64() > 0.0,
    ))
}

fn is_negative(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("negative?", args, 1)?;
    Ok(Value::Boolean(
        Numeric::from_value("negative?", &args[0])?.to_f64() < 0.0,
    ))
}

fn is_odd(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("odd?", args, 1)?;
    Ok(Value::Boolean(expect_number("odd?", &args[0])? % 2 != 0))
}

fn is_even(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("even?", args, 1)?;
    Ok(Value::Boolean(expect_number("even?", &args[0])? % 2 == 0))
}

fn max(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    let (first, rest) = args
        .split_first()
        .ok_or_else(|| SchemeError::arity("'max' expects at least 1 argument"))?;
    let mut current = Numeric::from_value("max", first)?;
    for value in rest {
        current = current.max(Numeric::from_value("max", value)?);
    }
    Ok(current.to_value())
}

fn min(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    let (first, rest) = args
        .split_first()
        .ok_or_else(|| SchemeError::arity("'min' expects at least 1 argument"))?;
    let mut current = Numeric::from_value("min", first)?;
    for value in rest {
        current = current.min(Numeric::from_value("min", value)?);
    }
    Ok(current.to_value())
}

fn quotient(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("quotient", args, 2)?;
    let dividend = expect_number("quotient", &args[0])?;
    let divisor = expect_number("quotient", &args[1])?;
    if divisor == 0 {
        return Err(SchemeError::runtime("division by zero"));
    }
    Ok(Value::Number(dividend / divisor))
}

fn remainder(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("remainder", args, 2)?;
    let dividend = expect_number("remainder", &args[0])?;
    let divisor = expect_number("remainder", &args[1])?;
    if divisor == 0 {
        return Err(SchemeError::runtime("division by zero"));
    }
    Ok(Value::Number(dividend % divisor))
}

fn modulo(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("modulo", args, 2)?;
    let dividend = expect_number("modulo", &args[0])?;
    let divisor = expect_number("modulo", &args[1])?;
    if divisor == 0 {
        return Err(SchemeError::runtime("division by zero"));
    }

    let remainder = dividend % divisor;
    let modulo =
        if remainder == 0 || (remainder > 0 && divisor > 0) || (remainder < 0 && divisor < 0) {
            remainder
        } else {
            remainder + divisor
        };
    Ok(Value::Number(modulo))
}

fn gcd(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    let mut current = 0_i64;
    for value in args {
        let number = expect_number("gcd", value)?;
        current = gcd_pair(current, number)?;
    }
    Ok(Value::Number(current))
}

fn lcm(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    let mut current = 1_i64;
    for value in args {
        let number = expect_number("lcm", value)?;
        current = lcm_pair(current, number)?;
    }
    Ok(Value::Number(current))
}

fn numeric_eq(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_numeric_chain("=", args, |left, right| {
        left == right
    })?))
}

fn numeric_lt(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_numeric_chain("<", args, |left, right| {
        left < right
    })?))
}

fn numeric_gt(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_numeric_chain(">", args, |left, right| {
        left > right
    })?))
}

fn numeric_lte(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_numeric_chain("<=", args, |left, right| {
        left <= right
    })?))
}

fn numeric_gte(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_numeric_chain(">=", args, |left, right| {
        left >= right
    })?))
}

fn number_to_string(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("number->string", args, 1)?;
    let text = match Numeric::from_value("number->string", &args[0])? {
        Numeric::Integer(value) => value.to_string(),
        Numeric::Float(value) => fmt_inexact_number(value),
    };
    Ok(Value::string(text))
}

fn string_to_number(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("string->number", args, 1)?;
    let text = expect_string("string->number", &args[0])?;
    let forms = match Reader::new(&text).read_all() {
        Ok(forms) => forms,
        Err(_) => return Ok(Value::Boolean(false)),
    };

    match forms.as_slice() {
        [Datum::Number(number)] => Ok(Value::Number(*number)),
        [Datum::Float(number)] => Ok(Value::Float(*number)),
        [_] | [] => Ok(Value::Boolean(false)),
        _ => Ok(Value::Boolean(false)),
    }
}

fn compare_numeric_chain<F>(name: &str, args: &[Value], cmp: F) -> Result<bool, SchemeError>
where
    F: Fn(f64, f64) -> bool,
{
    if args.len() < 2 {
        return Err(SchemeError::arity(format!(
            "'{name}' expects at least 2 arguments"
        )));
    }

    let mut previous = Numeric::from_value(name, &args[0])?.to_f64();
    for value in &args[1..] {
        let current = Numeric::from_value(name, value)?.to_f64();
        if !cmp(previous, current) {
            return Ok(false);
        }
        previous = current;
    }

    Ok(true)
}

fn integer_overflow(name: &str) -> SchemeError {
    SchemeError::runtime(format!("'{name}' integer overflow"))
}

fn checked_abs(name: &str, value: i64) -> Result<i64, SchemeError> {
    value
        .checked_abs()
        .ok_or_else(|| SchemeError::runtime(format!("'{name}' integer magnitude overflow")))
}

fn gcd_pair(left: i64, right: i64) -> Result<i64, SchemeError> {
    let mut a = checked_abs("gcd", left)?;
    let mut b = checked_abs("gcd", right)?;
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    Ok(a)
}

fn lcm_pair(left: i64, right: i64) -> Result<i64, SchemeError> {
    if left == 0 || right == 0 {
        return Ok(0);
    }

    let gcd = gcd_pair(left, right)?;
    let quotient = left / gcd;
    let product = quotient
        .checked_mul(right)
        .ok_or_else(|| SchemeError::runtime("'lcm' integer overflow"))?;
    checked_abs("lcm", product)
}
