use super::*;

pub(super) fn install(env: &mut Environment) {
    define_builtin(env, "number?", is_number);
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

fn is_number(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("number?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::Number(_))))
}

fn add(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    let mut sum = 0_i64;
    for value in args {
        sum += expect_number("+", value)?;
    }
    Ok(Value::Number(sum))
}

fn subtract(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    let (first, rest) = args
        .split_first()
        .ok_or_else(|| SchemeError::arity("'-' expects at least 1 argument"))?;
    let mut total = expect_number("-", first)?;
    if rest.is_empty() {
        return Ok(Value::Number(-total));
    }
    for value in rest {
        total -= expect_number("-", value)?;
    }
    Ok(Value::Number(total))
}

fn multiply(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    let mut product = 1_i64;
    for value in args {
        product *= expect_number("*", value)?;
    }
    Ok(Value::Number(product))
}

fn divide(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() < 2 {
        return Err(SchemeError::arity(
            "'/' in the minimal kernel expects at least 2 arguments",
        ));
    }
    let mut total = expect_number("/", &args[0])?;
    for value in &args[1..] {
        let divisor = expect_number("/", value)?;
        if divisor == 0 {
            return Err(SchemeError::runtime("division by zero"));
        }
        total /= divisor;
    }
    Ok(Value::Number(total))
}

fn abs(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("abs", args, 1)?;
    let number = expect_number("abs", &args[0])?;
    Ok(Value::Number(checked_abs("abs", number)?))
}

fn is_zero(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("zero?", args, 1)?;
    Ok(Value::Boolean(expect_number("zero?", &args[0])? == 0))
}

fn is_positive(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("positive?", args, 1)?;
    Ok(Value::Boolean(expect_number("positive?", &args[0])? > 0))
}

fn is_negative(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("negative?", args, 1)?;
    Ok(Value::Boolean(expect_number("negative?", &args[0])? < 0))
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
    let mut current = expect_number("max", first)?;
    for value in rest {
        current = current.max(expect_number("max", value)?);
    }
    Ok(Value::Number(current))
}

fn min(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    let (first, rest) = args
        .split_first()
        .ok_or_else(|| SchemeError::arity("'min' expects at least 1 argument"))?;
    let mut current = expect_number("min", first)?;
    for value in rest {
        current = current.min(expect_number("min", value)?);
    }
    Ok(Value::Number(current))
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
    Ok(Value::Boolean(compare_chain("=", args, |left, right| {
        left == right
    })?))
}

fn numeric_lt(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_chain("<", args, |left, right| {
        left < right
    })?))
}

fn numeric_gt(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_chain(">", args, |left, right| {
        left > right
    })?))
}

fn numeric_lte(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_chain("<=", args, |left, right| {
        left <= right
    })?))
}

fn numeric_gte(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_chain(">=", args, |left, right| {
        left >= right
    })?))
}

fn number_to_string(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("number->string", args, 1)?;
    let number = expect_number("number->string", &args[0])?;
    Ok(Value::string(number.to_string()))
}

fn string_to_number(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("string->number", args, 1)?;
    let text = expect_string("string->number", &args[0])?;
    match text.parse::<i64>() {
        Ok(number) => Ok(Value::Number(number)),
        Err(_) => Ok(Value::Boolean(false)),
    }
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
