use scheme4r::{eval, Value};

#[test]
fn datum_comments_skip_the_following_datum() {
    let value = eval(
        "\
        (list 1
              #;(+ 2 3)
              4)
        ",
    )
    .unwrap();
    assert_eq!(format!("{value}"), "(1 4)");

    let value = eval(
        "\
        #;(define ignored 99)
        42
        ",
    )
    .unwrap();
    assert!(matches!(value, Value::Number(42)));
}

#[test]
fn block_comments_support_nesting() {
    let value = eval(
        "\
        #| outer
           #| inner |#
        |#
        (+ 19 23)
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Number(42)));
}

#[test]
fn prefixed_integer_literals_work() {
    let value = eval("(list #b101010 #o52 #d42 #x2a #e42 #i42 #x#e2a #b-101010)").unwrap();
    assert_eq!(format!("{value}"), "(42 42 42 42 42 42 42 -42)");
}
