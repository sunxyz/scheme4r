use scheme4r::{eval, Value};

#[test]
fn character_literals_and_conversions_work() {
    let value = eval("#\\A").unwrap();
    assert!(matches!(value, Value::Character('A')));

    let value = eval("#\\space").unwrap();
    assert!(matches!(value, Value::Character(' ')));

    let value = eval("(char->integer #\\A)").unwrap();
    assert!(matches!(value, Value::Number(65)));

    let value = eval("(integer->char 66)").unwrap();
    assert!(matches!(value, Value::Character('B')));
}

#[test]
fn vectors_support_literals_conversion_and_mutation() {
    let value = eval("(vector-length #(1 2 3))").unwrap();
    assert!(matches!(value, Value::Number(3)));

    let value = eval("(vector-ref #(1 2 3) 1)").unwrap();
    assert!(matches!(value, Value::Number(2)));

    let value = eval(
        "\
        (let ((v (vector 1 2 3)))
          (vector-set! v 1 9)
          (vector->list v))
        ",
    )
    .unwrap();
    assert_eq!(format!("{value}"), "(1 9 3)");

    let value = eval("(list->vector '(4 5 6))").unwrap();
    assert_eq!(format!("{value}"), "#(4 5 6)");
}

#[test]
fn bytevectors_support_literals_and_mutation() {
    let value = eval("(bytevector-length #u8(1 2 255))").unwrap();
    assert!(matches!(value, Value::Number(3)));

    let value = eval("(bytevector-u8-ref #u8(1 2 255) 2)").unwrap();
    assert!(matches!(value, Value::Number(255)));

    let value = eval(
        "\
        (let ((bv (bytevector 1 2 3)))
          (bytevector-u8-set! bv 1 42)
          bv)
        ",
    )
    .unwrap();
    assert_eq!(format!("{value}"), "#u8(1 42 3)");
}

#[test]
fn quasiquote_supports_unquote_and_splicing_for_lists() {
    let value = eval("`(1 ,(+ 1 2) ,@(list 4 5) 6)").unwrap();
    assert_eq!(format!("{value}"), "(1 3 4 5 6)");
}

#[test]
fn quasiquote_supports_vectors_and_nested_unquote() {
    let value = eval("`#(1 ,(+ 1 2) ,@(list 4 5))").unwrap();
    assert_eq!(format!("{value}"), "#(1 3 4 5)");

    let value = eval("`(outer `(inner ,(+ 1 2)))").unwrap();
    assert_eq!(
        format!("{value}"),
        "(outer (quasiquote (inner (unquote (+ 1 2)))))"
    );
}
