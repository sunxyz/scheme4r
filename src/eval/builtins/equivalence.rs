use super::*;

pub(super) fn install(env: &mut Environment) {
    define_builtin(env, "eq?", eq);
    define_builtin(env, "eqv?", eqv);
    define_builtin(env, "equal?", equal);
}

fn eq(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("eq?", args, 2)?;
    Ok(Value::Boolean(Value::eqv(&args[0], &args[1])))
}

fn eqv(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("eqv?", args, 2)?;
    Ok(Value::Boolean(Value::eqv(&args[0], &args[1])))
}

fn equal(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("equal?", args, 2)?;
    Ok(Value::Boolean(Value::equal(&args[0], &args[1])))
}
