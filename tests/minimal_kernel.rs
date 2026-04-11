use scheme4r::{eval, interpreter, Environment, Value};

#[test]
fn arithmetic_and_nested_calls_work() {
    let value = eval("(+ 1 2 (* 3 4))").unwrap();
    assert!(matches!(value, Value::Number(15)));
}

#[test]
fn define_and_lambda_support_recursion() {
    let value = eval(
        "\
        (define (fact n)
          (if (= n 0)
              1
              (* n (fact (- n 1)))))
        (fact 6)
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Number(720)));
}

#[test]
fn quote_and_list_primitives_work() {
    let value = eval("(car '(1 2 3))").unwrap();
    assert!(matches!(value, Value::Number(1)));

    let list = eval("(cdr '(1 2 3))").unwrap();
    assert_eq!(format!("{list}"), "(2 3)");
}

#[test]
fn closures_capture_environment() {
    let value = eval(
        "\
        (define make-adder
          (lambda (x)
            (lambda (y) (+ x y))))
        (define add5 (make-adder 5))
        (add5 9)
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Number(14)));
}

#[test]
fn interpreter_reuses_injected_environment() {
    let env = Environment::standard();
    interpreter("(define counter 40)", env.clone()).unwrap();
    let value = interpreter("(begin (set! counter (+ counter 2)) counter)", env).unwrap();
    assert!(matches!(value, Value::Number(42)));
}

#[test]
fn equal_checks_structure() {
    let value = eval("(equal? '(1 (2 3)) '(1 (2 3)))").unwrap();
    assert!(matches!(value, Value::Boolean(true)));
}
