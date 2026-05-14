use scheme4r::{eval, ErrorKind, Value};

#[test]
fn boolean_and_symbol_equality_work() {
    let value = eval("(list (boolean=? #t #t #t) (symbol=? 'abc 'abc 'abc))").unwrap();
    assert_eq!(format!("{value}"), "(#t #t)");

    let value = eval("(list (boolean=? #t #f) (symbol=? 'abc 'xyz))").unwrap();
    assert_eq!(format!("{value}"), "(#f #f)");
}

#[test]
fn error_and_raise_surface_runtime_errors() {
    let err = eval("(error \"boom\" 1 'x)").unwrap_err();
    assert_eq!(err.kind, ErrorKind::Runtime);
    assert_eq!(err.message, "exception raised");
    let (object, continuable) = err.as_raised().unwrap();
    assert!(!continuable);
    match object {
        Value::ErrorObject(error) => {
            assert_eq!(error.message(), "boom");
            assert_eq!(
                format!("{}", Value::list(error.irritants().to_vec())),
                "(1 x)"
            );
        }
        other => panic!("expected error object, got {other}"),
    }

    let err = eval("(raise '(a b))").unwrap_err();
    assert_eq!(err.kind, ErrorKind::Runtime);
    assert_eq!(err.message, "exception raised");
    let (object, continuable) = err.as_raised().unwrap();
    assert!(!continuable);
    assert_eq!(format!("{object}"), "(a b)");
}

#[test]
fn values_and_call_with_values_work() {
    let value = eval(
        "\
        (call-with-values
          (lambda () (values 1 2 3))
          (lambda (a b c) (list a b c)))
        ",
    )
    .unwrap();
    assert_eq!(format!("{value}"), "(1 2 3)");

    let value = eval(
        "\
        (call-with-values
          (lambda () (values))
          (lambda () 'empty))
        ",
    )
    .unwrap();
    assert_eq!(format!("{value}"), "empty");

    let value = eval("(values 42)").unwrap();
    assert_eq!(format!("{value}"), "42");
}
