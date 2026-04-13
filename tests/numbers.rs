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
