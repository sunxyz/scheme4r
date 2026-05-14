use super::*;

pub(super) fn install(env: &mut Environment) {
    define_builtin(env, "symbol?", is_symbol);
    define_builtin(env, "symbol=?", symbol_equal);
    define_builtin(env, "symbol->string", symbol_to_string);
    define_builtin(env, "string->symbol", string_to_symbol);
}

fn is_symbol(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("symbol?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::Symbol(_))))
}

fn symbol_equal(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() {
        return Err(SchemeError::arity("'symbol=?' expects at least 1 argument"));
    }

    let mut previous = expect_symbol("symbol=?", &args[0])?.to_string();
    for value in &args[1..] {
        let current = expect_symbol("symbol=?", value)?;
        if current != previous {
            return Ok(Value::Boolean(false));
        }
        previous.clear();
        previous.push_str(current);
    }
    Ok(Value::Boolean(true))
}

fn symbol_to_string(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("symbol->string", args, 1)?;
    let symbol = expect_symbol("symbol->string", &args[0])?;
    Ok(Value::string(symbol.to_string()))
}

fn string_to_symbol(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("string->symbol", args, 1)?;
    let text = expect_string("string->symbol", &args[0])?;
    Ok(Value::symbol(text))
}
