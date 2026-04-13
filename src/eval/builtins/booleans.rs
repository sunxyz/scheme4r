use super::*;

pub(super) fn install(env: &mut Environment) {
    define_builtin(env, "boolean?", is_boolean);
    define_builtin(env, "boolean=?", boolean_equal);
    define_builtin(env, "not", not);
}

fn is_boolean(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("boolean?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::Boolean(_))))
}

fn boolean_equal(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() {
        return Err(SchemeError::arity(
            "'boolean=?' expects at least 1 argument",
        ));
    }

    let mut previous = expect_boolean("boolean=?", &args[0])?;
    for value in &args[1..] {
        let current = expect_boolean("boolean=?", value)?;
        if current != previous {
            return Ok(Value::Boolean(false));
        }
        previous = current;
    }
    Ok(Value::Boolean(true))
}

fn not(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("not", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::Boolean(false))))
}

fn expect_boolean(name: &str, value: &Value) -> Result<bool, SchemeError> {
    match value {
        Value::Boolean(boolean) => Ok(*boolean),
        other => Err(SchemeError::type_error(format!(
            "'{name}' expected a boolean, got {other}"
        ))),
    }
}
