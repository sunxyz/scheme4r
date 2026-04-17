use scheme4r::{eval, Value};

#[test]
fn dict_creation_lookup_and_keys_work() {
    let value = eval(
        "\
        (let ((d (dict 'b 2 'a 1 #t 3)))
          (list (dict? d)
                (dict-size d)
                (dict-ref d 'a)
                (dict-has-key? d #t)
                (dict-has-key? d 'missing)
                (dict-keys d)))
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(#t 3 1 #t #f (#t a b))");
}

#[test]
fn dict_mutation_and_default_value_work() {
    let value = eval(
        "\
        (let ((d (make-dict)))
          (dict-set! d 'x 10)
          (dict-set! d 'x 20)
          (list (dict-size d)
                (dict-ref d 'x)
                (dict-delete! d 'x)
                (dict-size d)
                (dict-ref d 'x 99)))
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(1 20 #t 0 99)");
}

#[test]
fn dict_rejects_unsupported_key_types() {
    let err = eval(
        "\
        (let ((d (make-dict)))
          (dict-set! d '(1 2) 3))
        ",
    )
    .unwrap_err();

    assert!(err
        .message
        .contains("key must be boolean/number/character/string/symbol/()"));
    let value = eval(
        "\
        (let ((d (dict 'a 1)))
          (dict-clear! d)
          (dict-size d))
        ",
    )
    .unwrap();
    assert!(matches!(value, Value::Number(0)));
}
