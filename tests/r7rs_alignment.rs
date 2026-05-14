use scheme4r::{eval, ErrorKind, Value};

#[test]
fn case_lambda_dispatches_on_arity() {
    let value = eval(
        "\
        (define chooser
          (case-lambda
            (() 'zero)
            ((x) x)
            ((x y . rest) (list x y rest))))
        (list (chooser) (chooser 42) (chooser 1 2 3 4))
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(zero 42 (1 2 (3 4)))");
}

#[test]
fn define_values_and_let_values_bind_multiple_values() {
    let value = eval(
        "\
        (define-values (a b) (values 10 20))
        (let-values (((x y) (values 1 2))
                     ((z) (values 3)))
          (let*-values (((sum) (values (+ x y z)))
                        ((left right) (values a b)))
            (list sum left right)))
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(6 10 20)");
}

#[test]
fn eval_accepts_environment_specifier() {
    let value = eval("(eval '(+ 19 23) (environment '(scheme base)))").unwrap();
    assert!(matches!(value, Value::Number(42)));
}

#[test]
fn environment_specifier_uses_import_sets_instead_of_current_env() {
    let err = eval(
        "\
        (define hidden 41)
        (eval 'hidden (environment '(scheme base)))
        ",
    )
    .unwrap_err();

    assert_eq!(err.kind, ErrorKind::Name);
}

#[test]
fn scheme_base_environment_does_not_expose_write_bindings() {
    let err = eval("(eval 'display (environment '(scheme base)))").unwrap_err();
    assert_eq!(err.kind, ErrorKind::Name);

    let value = eval(
        "\
        (procedure? (eval 'display (environment '(scheme write))))
        ",
    )
    .unwrap();
    assert!(matches!(value, Value::Boolean(true)));
}
