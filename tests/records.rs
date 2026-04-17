use scheme4r::{eval, Value};

#[test]
fn define_record_type_generates_constructor_predicate_accessor_and_mutator() {
    let value = eval(
        "\
        (define-record-type <point>
          (make-point x y)
          point?
          (x point-x set-point-x!)
          (y point-y))

        (let ((p (make-point 1 2)))
          (list (point? p)
                (point-x p)
                (point-y p)
                (begin
                  (set-point-x! p 9)
                  (point-x p))))
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(#t 1 2 9)");
}

#[test]
fn record_predicate_and_accessor_handle_type_mismatch() {
    let value = eval(
        "\
        (define-record-type <point>
          (make-point x y)
          point?
          (x point-x)
          (y point-y))

        (define-record-type <size>
          (make-size w h)
          size?
          (w size-w)
          (h size-h))

        (let ((s (make-size 3 4)))
          (point? s))
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Boolean(false)));

    let err = eval(
        "\
        (define-record-type <point>
          (make-point x y)
          point?
          (x point-x)
          (y point-y))

        (define-record-type <size>
          (make-size w h)
          size?
          (w size-w)
          (h size-h))

        (let ((s (make-size 3 4)))
          (point-x s))
        ",
    )
    .unwrap_err();

    assert!(err.message.contains("matching record type"));
}
