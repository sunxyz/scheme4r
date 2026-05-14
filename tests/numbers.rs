use scheme4r::{eval, Value};

#[test]
fn numeric_predicates_and_extrema_work() {
    let value = eval(
        "\
        (list (abs -7)
              (zero? 0)
              (positive? 5)
              (negative? -2)
              (odd? 9)
              (even? 12)
              (max 4 9 2 7)
              (min 4 9 2 7))
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(7 #t #t #t #t #t 9 2)");
}

#[test]
fn quotient_remainder_and_modulo_follow_integer_semantics() {
    let value = eval(
        "\
        (list (quotient -13 4)
              (remainder -13 4)
              (modulo -13 4)
              (quotient 13 -4)
              (remainder 13 -4)
              (modulo 13 -4))
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(-3 -1 3 -3 1 -3)");
}

#[test]
fn gcd_and_lcm_support_variadic_integer_inputs() {
    let value = eval("(list (gcd 32 -36 48) (lcm 4 6 14) (gcd) (lcm))").unwrap();
    assert_eq!(format!("{value}"), "(4 84 0 1)");

    let value = eval("(lcm 0 5 10)").unwrap();
    assert!(matches!(value, Value::Number(0)));
}

#[test]
fn mixed_numeric_operations_promote_to_inexact_when_needed() {
    let value = eval(
        "\
        (list (+ 1 2.5)
              (- 5 1.5)
              (* 2 1.5)
              (/ 3 2)
              (/ 6 3)
              (/ 4)
              (max 1 2.5 2)
              (min 1.5 2 3))
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(3.5 3.5 3.0 1.5 2 0.25 2.5 1.5)");
}

#[test]
fn numeric_type_predicates_distinguish_exact_and_inexact_values() {
    let value = eval(
        "\
        (list (number? 1.5)
              (complex? 1.5)
              (real? 1.5)
              (rational? 1.5)
              (integer? 3.0)
              (integer? 3.5)
              (exact? 3)
              (exact? 3.0)
              (inexact? 3.0)
              (exact-integer? 3)
              (exact-integer? 3.0)
              (finite? 3)
              (infinite? 3.0)
              (nan? 3.0))
        ",
    )
    .unwrap();

    assert_eq!(
        format!("{value}"),
        "(#t #t #t #t #t #f #t #f #t #t #f #t #f #f)"
    );
}

#[test]
fn string_to_number_and_number_to_string_support_inexact_numbers() {
    let value = eval(
        "\
        (list (string->number \"42\")
              (string->number \"3.5\")
              (string->number \"#x2a\")
              (string->number \"not-a-number\")
              (number->string 2.0)
              (number->string 0.25))
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(42 3.5 42 #f \"2.0\" \"0.25\")");
}
