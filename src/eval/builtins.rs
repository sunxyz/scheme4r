use crate::{
    error::SchemeError,
    runtime::{environment::Environment, Value},
};

pub(crate) fn install(env: &mut Environment) {
    define_builtin(env, "+", add);
    define_builtin(env, "-", subtract);
    define_builtin(env, "*", multiply);
    define_builtin(env, "/", divide);
    define_builtin(env, "=", numeric_eq);
    define_builtin(env, "<", numeric_lt);
    define_builtin(env, ">", numeric_gt);
    define_builtin(env, "<=", numeric_lte);
    define_builtin(env, ">=", numeric_gte);
    define_builtin(env, "cons", cons);
    define_builtin(env, "car", car);
    define_builtin(env, "cdr", cdr);
    define_builtin(env, "list", list);
    define_builtin(env, "null?", is_null);
    define_builtin(env, "pair?", is_pair);
    define_builtin(env, "list?", is_list);
    define_builtin(env, "number?", is_number);
    define_builtin(env, "boolean?", is_boolean);
    define_builtin(env, "symbol?", is_symbol);
    define_builtin(env, "string?", is_string);
    define_builtin(env, "procedure?", is_procedure);
    define_builtin(env, "eq?", eq);
    define_builtin(env, "equal?", equal);
    define_builtin(env, "not", not);
    define_builtin(env, "display", display);
    define_builtin(env, "newline", newline);
}

fn define_builtin(
    env: &mut Environment,
    name: &'static str,
    func: fn(&[Value]) -> Result<Value, SchemeError>,
) {
    env.define(name, Value::builtin(name, func));
}

fn add(args: &[Value]) -> Result<Value, SchemeError> {
    let mut sum = 0_i64;
    for value in args {
        sum += expect_number("+", value)?;
    }
    Ok(Value::Number(sum))
}

fn subtract(args: &[Value]) -> Result<Value, SchemeError> {
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

fn multiply(args: &[Value]) -> Result<Value, SchemeError> {
    let mut product = 1_i64;
    for value in args {
        product *= expect_number("*", value)?;
    }
    Ok(Value::Number(product))
}

fn divide(args: &[Value]) -> Result<Value, SchemeError> {
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

fn numeric_eq(args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_chain("=", args, |left, right| {
        left == right
    })?))
}

fn numeric_lt(args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_chain("<", args, |left, right| {
        left < right
    })?))
}

fn numeric_gt(args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_chain(">", args, |left, right| {
        left > right
    })?))
}

fn numeric_lte(args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_chain("<=", args, |left, right| {
        left <= right
    })?))
}

fn numeric_gte(args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_chain(">=", args, |left, right| {
        left >= right
    })?))
}

fn cons(args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("cons", args, 2)?;
    Ok(Value::pair(args[0].clone(), args[1].clone()))
}

fn car(args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("car", args, 1)?;
    let pair = expect_pair("car", &args[0])?;
    let value = pair.borrow().car.clone();
    Ok(value)
}

fn cdr(args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("cdr", args, 1)?;
    let pair = expect_pair("cdr", &args[0])?;
    let value = pair.borrow().cdr.clone();
    Ok(value)
}

fn list(args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::list(args.to_vec()))
}

fn is_null(args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("null?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::EmptyList)))
}

fn is_pair(args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("pair?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::Pair(_))))
}

fn is_list(args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("list?", args, 1)?;
    Ok(Value::Boolean(is_proper_list(&args[0])))
}

fn is_number(args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("number?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::Number(_))))
}

fn is_boolean(args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("boolean?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::Boolean(_))))
}

fn is_symbol(args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("symbol?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::Symbol(_))))
}

fn is_string(args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("string?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::String(_))))
}

fn is_procedure(args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("procedure?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::Procedure(_))))
}

fn eq(args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("eq?", args, 2)?;
    Ok(Value::Boolean(Value::eqv(&args[0], &args[1])))
}

fn equal(args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("equal?", args, 2)?;
    Ok(Value::Boolean(Value::equal(&args[0], &args[1])))
}

fn not(args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("not", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::Boolean(false))))
}

fn display(args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("display", args, 1)?;
    match &args[0] {
        Value::String(text) => print!("{text}"),
        value => print!("{value}"),
    }
    Ok(Value::Unspecified)
}

fn newline(args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("newline", args, 0)?;
    println!();
    Ok(Value::Unspecified)
}

fn expect_arity(name: &str, args: &[Value], expected: usize) -> Result<(), SchemeError> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(SchemeError::arity(format!(
            "'{name}' expects {expected} arguments, got {}",
            args.len()
        )))
    }
}

fn expect_number(name: &str, value: &Value) -> Result<i64, SchemeError> {
    match value {
        Value::Number(number) => Ok(*number),
        other => Err(SchemeError::type_error(format!(
            "'{name}' expected a number, got {other}"
        ))),
    }
}

fn expect_pair<'a>(
    name: &str,
    value: &'a Value,
) -> Result<std::rc::Rc<std::cell::RefCell<crate::runtime::pair::PairCell>>, SchemeError> {
    match value {
        Value::Pair(pair) => Ok(pair.clone()),
        other => Err(SchemeError::type_error(format!(
            "'{name}' expected a pair, got {other}"
        ))),
    }
}

fn compare_chain<F>(name: &str, args: &[Value], cmp: F) -> Result<bool, SchemeError>
where
    F: Fn(i64, i64) -> bool,
{
    if args.len() < 2 {
        return Err(SchemeError::arity(format!(
            "'{name}' expects at least 2 arguments"
        )));
    }

    let mut previous = expect_number(name, &args[0])?;
    for value in &args[1..] {
        let current = expect_number(name, value)?;
        if !cmp(previous, current) {
            return Ok(false);
        }
        previous = current;
    }
    Ok(true)
}

fn is_proper_list(value: &Value) -> bool {
    let mut current = value.clone();
    loop {
        match current {
            Value::EmptyList => return true,
            Value::Pair(pair) => {
                current = pair.borrow().cdr.clone();
            }
            _ => return false,
        }
    }
}
