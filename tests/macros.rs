use scheme4r::{eval, Value};

#[test]
fn define_syntax_when_expands_and_runs() {
    let value = eval(
        "\
        (define x 0)
        (define-syntax when
          (syntax-rules ()
            ((_ test body ...)
             (if test (begin body ...)))))
        (when #t
          (set! x 41)
          (set! x (+ x 1)))
        x
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Number(42)));
}

#[test]
fn syntax_rules_support_recursive_variadic_macros() {
    let value = eval(
        "\
        (define-syntax and
          (syntax-rules ()
            ((_ ) #t)
            ((_ x) x)
            ((_ x y ...) (if x (and y ...) x))))
        (and #t 1 2 3)
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Number(3)));
}

#[test]
fn syntax_rules_support_literal_identifiers() {
    let value = eval(
        "\
        (define-syntax my-cond
          (syntax-rules (else)
            ((_ (else expr)) expr)
            ((_ (test expr) rest ...)
             (if test expr (my-cond rest ...)))))
        (my-cond ((> 1 2) 11) (else 99))
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Number(99)));
}

#[test]
fn macros_can_expand_into_bindings_and_lists() {
    let value = eval(
        "\
        (define-syntax push-front
          (syntax-rules ()
            ((_ item lst)
             (set! lst (cons item lst)))))
        (define values '(2 3))
        (push-front 1 values)
        values
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(1 2 3)");
}

#[test]
fn let_syntax_creates_local_macro_scope() {
    let value = eval(
        "\
        (let-syntax
          ((swap
             (syntax-rules ()
               ((_ a b)
                (let ((tmp a))
                  (set! a b)
                  (set! b tmp))))))
          (let ((x 1) (y 2))
            (swap x y)
            (list x y)))
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(2 1)");
    assert!(eval("(swap 1 2)").is_err());
}

#[test]
fn letrec_syntax_supports_local_recursive_macros() {
    let value = eval(
        "\
        (letrec-syntax
          ((my-or
             (syntax-rules ()
               ((_ ) #f)
               ((_ x) x)
               ((_ x y ...) (let ((tmp x)) (if tmp tmp (my-or y ...)))))))
          (my-or #f #f 7 #f))
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Number(7)));
}
