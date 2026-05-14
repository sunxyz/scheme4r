use scheme4r::{eval, Value};

#[test]
fn tail_recursive_self_call_returns_expected_result() {
    let value = eval(
        "\
        (define (loop n acc)
          (if (= n 0)
              acc
              (loop (- n 1) (+ acc 1))))
        (loop 25 0)
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Number(25)));
}

#[test]
fn named_let_tail_call_returns_expected_result() {
    let value = eval(
        "\
        (let loop ((n 10) (acc 0))
          (if (= n 0)
              acc
              (loop (- n 1) (+ acc 2))))
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Number(20)));
}

#[test]
fn mutual_tail_recursion_returns_expected_result() {
    let value = eval(
        "\
        (letrec ((even?
                  (lambda (n)
                    (if (= n 0) #t (odd? (- n 1)))))
                 (odd?
                  (lambda (n)
                    (if (= n 0) #f (even? (- n 1))))))
          (list (even? 12) (odd? 13)))
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(#t #t)");
}

#[test]
fn begin_preserves_tail_call_result() {
    let value = eval(
        "\
        (define (loop n)
          (begin
            'ignored
            (if (= n 0)
                'done
                (loop (- n 1)))))
        (loop 8)
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "done");
}

#[test]
fn cond_and_case_preserve_tail_call_result() {
    let cond_value = eval(
        "\
        (define (loop-cond n)
          (cond
            ((= n 0) 'done)
            (else (loop-cond (- n 1)))))
        (loop-cond 9)
        ",
    )
    .unwrap();
    assert_eq!(format!("{cond_value}"), "done");

    let case_value = eval(
        "\
        (define (loop-case n)
          (case (= n 0)
            ((#t) 'done)
            (else (loop-case (- n 1)))))
        (loop-case 9)
        ",
    )
    .unwrap();
    assert_eq!(format!("{case_value}"), "done");
}

#[test]
fn deep_tail_recursion_self_call_completes() {
    let value = eval(
        "\
        (define (loop n acc)
          (if (= n 0)
              acc
              (loop (- n 1) (+ acc 1))))
        (loop 100000 0)
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Number(100000)));
}

#[test]
fn deep_named_let_completes() {
    let value = eval(
        "\
        (let loop ((n 100000) (acc 0))
          (if (= n 0)
              acc
              (loop (- n 1) (+ acc 1))))
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Number(100000)));
}

#[test]
fn deep_tail_recursion_through_apply_completes() {
    let value = eval(
        "\
        (define (loop n)
          (if (= n 0)
              0
              (apply loop (list (- n 1)))))
        (loop 100000)
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Number(0)));
}

#[test]
fn deep_tail_recursion_through_call_with_values_completes() {
    let value = eval(
        "\
        (define (loop n)
          (if (= n 0)
              0
              (call-with-values
                (lambda () (values (- n 1)))
                loop)))
        (loop 100000)
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Number(0)));
}

#[test]
fn deep_tail_recursion_through_eval_completes() {
    let value = eval(
        "\
        (define (loop n)
          (if (= n 0)
              0
              (eval (list 'loop (- n 1)))))
        (loop 20000)
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Number(0)));
}

#[test]
fn deep_tail_recursion_through_call_cc_completes() {
    let value = eval(
        "\
        (define (loop n)
          (if (= n 0)
              0
              (call/cc (lambda (k) (loop (- n 1))))))
        (loop 100000)
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Number(0)));
}

#[test]
fn deep_tail_recursion_through_dynamic_wind_completes() {
    let value = eval(
        "\
        (define (loop n)
          (if (= n 0)
              0
              (dynamic-wind
                (lambda () 'before)
                (lambda () (loop (- n 1)))
                (lambda () 'after))))
        (loop 30000)
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Number(0)));
}

#[test]
fn deep_tail_recursion_through_with_exception_handler_completes() {
    let value = eval(
        "\
        (define (loop n)
          (if (= n 0)
              0
              (with-exception-handler
                (lambda (obj) obj)
                (lambda () (loop (- n 1))))))
        (loop 30000)
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Number(0)));
}
