use scheme4r::{Environment, Scheme, Value};

#[test]
fn scheme_object_reuses_state_across_eval_calls() {
    let scheme = Scheme::new(Environment::standard());

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
