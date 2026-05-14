use std::collections::HashMap;

use scheme4r::{eval::Engine, BuiltinFn, Environment, ErrorKind, Scheme, SchemeError, Value};

fn host_double(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() != 1 {
        return Err(SchemeError::new(
            ErrorKind::Arity,
            "'double' expects exactly 1 argument",
            None,
        ));
    }

    match &args[0] {
        Value::Number(number) => Ok(Value::Number(number * 2)),
        other => Err(SchemeError::new(
            ErrorKind::Type,
            format!("'double' expected a number, got {other}"),
            None,
        )),
    }
}

#[test]
fn scheme_object_reuses_state_across_eval_calls() {
    let scheme = Scheme::with_env(Environment::standard());

    scheme.eval("(define counter 40)").unwrap();

    let value = scheme
        .eval("(begin (set! counter (+ counter 2)) counter)")
        .unwrap();
    assert!(matches!(value, Value::Number(42)));

    let value = scheme.eval("counter").unwrap();
    assert!(matches!(value, Value::Number(42)));
}

#[test]
fn default_scheme_uses_standard_environment() {
    let scheme = Scheme::default();
    let value = scheme.eval("(+ 1 2 3)").unwrap();
    assert!(matches!(value, Value::Number(6)));
}

#[test]
fn scheme_new_accepts_host_builtin_registry() {
    let mut builtins = HashMap::new();
    builtins.insert("double".to_string(), host_double as BuiltinFn);

    let scheme = Scheme::new(builtins);
    let value = scheme.eval("(double 21)").unwrap();
    assert!(matches!(value, Value::Number(42)));
}
