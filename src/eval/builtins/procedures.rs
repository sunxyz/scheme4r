use super::*;

pub(super) fn install(env: &mut Environment) {
    define_builtin(env, "procedure?", is_procedure);
    define_builtin(env, "apply", apply);
}

fn is_procedure(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("procedure?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::Procedure(_))))
}

fn apply(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() < 2 {
        return Err(SchemeError::arity(
            "'apply' expects a procedure, optional arguments, and a final list",
        ));
    }

    let procedure = args[0].clone();
    let (last, leading) = args[1..].split_last().ok_or_else(|| {
        SchemeError::arity("'apply' expects a procedure and a final argument list")
    })?;

    let mut applied_args = leading.to_vec();
    applied_args.extend(expect_list("apply", last)?);
    engine.apply(procedure, engine.current_env(), applied_args)
}
