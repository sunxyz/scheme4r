use super::*;
use crate::{
    error::SchemeError,
    runtime::{
        error_object::{ErrorObject, ErrorObjectKind},
        parameter::ParameterObject,
    },
};

pub(super) fn install(env: &mut Environment) {
    define_builtin(
        env,
        "call-with-current-continuation",
        call_with_current_continuation,
    );
    define_builtin(env, "call/cc", call_with_current_continuation);
    define_builtin(env, "dynamic-wind", dynamic_wind);
    define_builtin(env, "with-exception-handler", with_exception_handler);
    define_builtin(env, "error", error);
    define_builtin(env, "raise", raise);
    define_builtin(env, "raise-continuable", raise_continuable);
    define_builtin(env, "error-object?", error_object);
    define_builtin(env, "error-object-message", error_object_message);
    define_builtin(env, "error-object-irritants", error_object_irritants);
    define_builtin(env, "read-error?", read_error);
    define_builtin(env, "file-error?", file_error);
    define_builtin(env, "make-parameter", make_parameter);
    define_builtin(env, "values", values);
    define_builtin(env, "call-with-values", call_with_values);
}

fn error(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    let (message, irritants) = args
        .split_first()
        .ok_or_else(|| SchemeError::arity("'error' expects at least 1 argument"))?;
    let message = expect_string("error", message)?;
    Err(SchemeError::raised(
        Value::error_object(ErrorObject::new(
            ErrorObjectKind::General,
            message,
            irritants.to_vec(),
        )),
        false,
    ))
}

fn raise(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("raise", args, 1)?;
    Err(SchemeError::raised(args[0].clone(), false))
}

fn raise_continuable(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("raise-continuable", args, 1)?;
    Err(SchemeError::raised(args[0].clone(), true))
}

fn values(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(match args {
        [] => Value::multiple(Vec::new()),
        [value] => value.clone(),
        _ => Value::multiple(args.to_vec()),
    })
}

fn call_with_values(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("call-with-values", args, 2)?;
    let producer = args[0].clone();
    let consumer = args[1].clone();

    let produced = engine.apply(producer, engine.current_env(), Vec::new())?;
    let consumer_args = match produced {
        Value::Multiple(values) => values,
        value => vec![value],
    };

    engine.apply(consumer, engine.current_env(), consumer_args)
}

fn call_with_current_continuation(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("call-with-current-continuation", args, 1)?;
    let token = next_continuation_token();
    let continuation = Value::Continuation(token);
    match engine.apply(args[0].clone(), engine.current_env(), vec![continuation]) {
        Ok(value) => Ok(value),
        Err(err) => match err.as_continuation_jump() {
            Some((jump_token, value)) if jump_token == token => Ok(value.clone()),
            _ => Err(err),
        },
    }
}

fn dynamic_wind(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("dynamic-wind", args, 3)?;
    engine.apply(args[0].clone(), engine.current_env(), Vec::new())?;
    let result = engine.apply(args[1].clone(), engine.current_env(), Vec::new());
    let after = engine.apply(args[2].clone(), engine.current_env(), Vec::new());
    match (result, after) {
        (Ok(value), Ok(_)) => Ok(value),
        (Err(err), Ok(_)) => Err(err),
        (_, Err(err)) => Err(err),
    }
}

fn with_exception_handler(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("with-exception-handler", args, 2)?;
    match engine.apply(args[1].clone(), engine.current_env(), Vec::new()) {
        Ok(value) => Ok(value),
        Err(err) => {
            if let Some((object, continuable)) = err.as_raised() {
                let handled =
                    engine.apply(args[0].clone(), engine.current_env(), vec![object.clone()])?;
                if continuable {
                    Ok(handled)
                } else {
                    Err(SchemeError::raised(handled, false))
                }
            } else {
                Err(err)
            }
        }
    }
}

fn error_object(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("error-object?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::ErrorObject(_))))
}

fn error_object_message(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("error-object-message", args, 1)?;
    match &args[0] {
        Value::ErrorObject(error) => Ok(Value::string(error.message())),
        other => Err(SchemeError::type_error(format!(
            "'error-object-message' expected an error object, got {other}"
        ))),
    }
}

fn error_object_irritants(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("error-object-irritants", args, 1)?;
    match &args[0] {
        Value::ErrorObject(error) => Ok(Value::list(error.irritants().to_vec())),
        other => Err(SchemeError::type_error(format!(
            "'error-object-irritants' expected an error object, got {other}"
        ))),
    }
}

fn read_error(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("read-error?", args, 1)?;
    Ok(Value::Boolean(matches!(
        &args[0],
        Value::ErrorObject(error) if error.kind() == &ErrorObjectKind::Read
    )))
}

fn file_error(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("file-error?", args, 1)?;
    Ok(Value::Boolean(matches!(
        &args[0],
        Value::ErrorObject(error) if error.kind() == &ErrorObjectKind::File
    )))
}

fn make_parameter(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 2 {
        return Err(SchemeError::arity(
            "'make-parameter' expects 1 or 2 arguments",
        ));
    }
    let parameter = ParameterObject::new(args[0].clone(), args.get(1).cloned());
    Ok(Value::parameter(parameter))
}

fn next_continuation_token() -> usize {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT_TOKEN: AtomicUsize = AtomicUsize::new(1);
    NEXT_TOKEN.fetch_add(1, Ordering::Relaxed)
}
