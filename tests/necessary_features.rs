use scheme4r::{eval, Value};

#[test]
fn and_and_or_short_circuit_correctly() {
    let value = eval("(and #t 1 2 3)").unwrap();
    assert!(matches!(value, Value::Number(3)));

    let value = eval("(or #f #f 9)").unwrap();
    assert!(matches!(value, Value::Number(9)));

    let value = eval(
        "\
        (define touched 0)
        (or #t (set! touched 1))
        touched
        ",
    )
    .unwrap();
    assert!(matches!(value, Value::Number(0)));
}

#[test]
fn let_variants_work_for_parallel_sequential_and_recursive_bindings() {
    let value = eval("(let ((x 2) (y 3)) (+ x y))").unwrap();
    assert!(matches!(value, Value::Number(5)));

    let value = eval("(let* ((x 2) (y (+ x 3))) (* x y))").unwrap();
    assert!(matches!(value, Value::Number(10)));

    let value = eval(
        "\
        (letrec ((even?
                  (lambda (n)
                    (if (= n 0) #t (odd? (- n 1)))))
                 (odd?
                  (lambda (n)
                    (if (= n 0) #f (even? (- n 1))))))
          (even? 8))
        ",
    )
    .unwrap();
    assert!(matches!(value, Value::Boolean(true)));
}

#[test]
fn named_let_supports_iteration() {
    let value = eval(
        "\
        (let loop ((n 5) (acc 1))
          (if (= n 0)
              acc
              (loop (- n 1) (* acc n))))
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Number(120)));
}

#[test]
fn cond_supports_plain_else_and_arrow_clauses() {
    let value = eval(
        "\
        (cond
          ((> 1 2) 0)
          ((+ 1 2) => (lambda (x) (+ x 10)))
          (else 99))
        ",
    )
    .unwrap();
    assert!(matches!(value, Value::Number(13)));

    let value = eval(
        "\
        (cond
          ((> 1 2) 0)
          (else 99))
        ",
    )
    .unwrap();
    assert!(matches!(value, Value::Number(99)));
}

#[test]
fn case_and_do_forms_work() {
    let value = eval(
        "\
        (case (* 2 3)
          ((1 2 3) 'small)
          ((6 7) => (lambda (x) (+ x 10)))
          (else 'other))
        ",
    )
    .unwrap();
    assert!(matches!(value, Value::Number(16)));

    let value = eval(
        "\
        (do ((i 0 (+ i 1))
             (acc 0 (+ acc i)))
            ((= i 5) acc))
        ",
    )
    .unwrap();
    assert!(matches!(value, Value::Number(10)));
}

#[test]
fn internal_definitions_work_inside_bodies() {
    let value = eval(
        "\
        ((lambda ()
           (define even?
             (lambda (n)
               (if (= n 0) #t (odd? (- n 1)))))
           (define odd?
             (lambda (n)
               (if (= n 0) #f (even? (- n 1)))))
           (even? 8)))
        ",
    )
    .unwrap();
    assert!(matches!(value, Value::Boolean(true)));

    let value = eval(
        "\
        (let ()
          (define x 10)
          (define y (+ x 5))
          (+ x y))
        ",
    )
    .unwrap();
    assert!(matches!(value, Value::Number(25)));
}

#[test]
fn list_library_functions_work() {
    let value = eval("(length '(1 2 3 4))").unwrap();
    assert!(matches!(value, Value::Number(4)));

    let value = eval("(append '(1 2) '(3 4))").unwrap();
    assert_eq!(format!("{value}"), "(1 2 3 4)");

    let value = eval("(reverse '(1 2 3))").unwrap();
    assert_eq!(format!("{value}"), "(3 2 1)");

    let value = eval("(list-ref '(10 20 30) 1)").unwrap();
    assert!(matches!(value, Value::Number(20)));

    let value = eval("(list-tail '(10 20 30) 2)").unwrap();
    assert_eq!(format!("{value}"), "(30)");
}

#[test]
fn apply_and_string_conversions_work() {
    let value = eval("(apply + '(1 2 3 4))").unwrap();
    assert!(matches!(value, Value::Number(10)));

    let value = eval("(apply list 1 2 '(3 4))").unwrap();
    assert_eq!(format!("{value}"), "(1 2 3 4)");

    let value = eval("(string-append \"hel\" \"lo\")").unwrap();
    assert_eq!(format!("{value}"), "\"hello\"");

    let value = eval("(string-length \"hello\")").unwrap();
    assert!(matches!(value, Value::Number(5)));

    let value = eval("(string=? (symbol->string 'abc) \"abc\")").unwrap();
    assert!(matches!(value, Value::Boolean(true)));

    let value = eval("(string->symbol \"xyz\")").unwrap();
    assert_eq!(format!("{value}"), "xyz");

    let value = eval("(string->number \"42\")").unwrap();
    assert!(matches!(value, Value::Number(42)));
}
