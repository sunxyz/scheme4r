use scheme4r::{eval, Value};

#[test]
fn vectors_support_fill_copy_append_and_in_place_copy() {
    let value = eval(
        "\
        (let ((v (vector 1 2 3 4)))
          (vector-fill! v 'x 1 3)
          v)
        ",
    )
    .unwrap();
    assert_eq!(format!("{value}"), "#(1 x x 4)");

    let value = eval("(vector-copy #(1 2 3 4 5) 1 4)").unwrap();
    assert_eq!(format!("{value}"), "#(2 3 4)");

    let value = eval("(vector-append #(1 2) #(3) #(4 5))").unwrap();
    assert_eq!(format!("{value}"), "#(1 2 3 4 5)");

    let value = eval(
        "\
        (let ((v (vector 0 1 2 3 4)))
          (vector-copy! v 1 v 2 5)
          v)
        ",
    )
    .unwrap();
    assert_eq!(format!("{value}"), "#(0 2 3 4 4)");

    let value = eval("(vector-map (lambda (x) (* x x)) #(1 2 3))").unwrap();
    assert_eq!(format!("{value}"), "#(1 4 9)");

    let value = eval("(vector-map + #(1 2 3) #(10 20 30))").unwrap();
    assert_eq!(format!("{value}"), "#(11 22 33)");

    let value = eval(
        "\
        (let ((p (open-output-string)))
          (vector-for-each (lambda (x y) (display (+ x y) p))
                           #(1 2 3)
                           #(4 5 6))
          (get-output-string p))
        ",
    )
    .unwrap();
    assert!(matches!(value, Value::String(text) if text == "579"));
}

#[test]
fn bytevectors_support_copy_append_and_utf8_conversion() {
    let value = eval("(bytevector-copy #u8(10 20 30 40) 1 3)").unwrap();
    assert_eq!(format!("{value}"), "#u8(20 30)");

    let value = eval("(bytevector-append #u8(1 2) #u8(3) #u8(4 5))").unwrap();
    assert_eq!(format!("{value}"), "#u8(1 2 3 4 5)");

    let value = eval(
        "\
        (let ((bv (bytevector 1 2 3 4 5)))
          (bytevector-copy! bv 0 bv 2 5)
          bv)
        ",
    )
    .unwrap();
    assert_eq!(format!("{value}"), "#u8(3 4 5 4 5)");

    let value = eval("(string->utf8 \"A中B\")").unwrap();
    assert_eq!(format!("{value}"), "#u8(65 228 184 173 66)");

    let value = eval("(utf8->string #u8(65 228 184 173 66))").unwrap();
    assert!(matches!(value, Value::String(text) if text == "A中B"));

    let value = eval("(utf8->string #u8(65 228 184 173 66) 1 4)").unwrap();
    assert!(matches!(value, Value::String(text) if text == "中"));
}

#[test]
fn bytevector_ports_and_in_place_reads_work() {
    let value = eval(
        "\
        (let ((p (open-input-bytevector #u8(1 2 3 4))))
          (list (read-bytevector 2 p)
                (read-bytevector 3 p)
                (eof-object? (read-bytevector 1 p))))
        ",
    )
    .unwrap();
    assert_eq!(format!("{value}"), "(#u8(1 2) #u8(3 4) #t)");

    let value = eval(
        "\
        (let ((p (open-input-bytevector #u8(10 20 30 40)))
              (bv (make-bytevector 5 0)))
          (list (read-bytevector! bv p 1 4)
                bv))
        ",
    )
    .unwrap();
    assert_eq!(format!("{value}"), "(3 #u8(0 10 20 30 0))");

    let value = eval(
        "\
        (let ((p (open-output-bytevector)))
          (write-bytevector #u8(1 2 3 4) p 1 3)
          (get-output-bytevector p))
        ",
    )
    .unwrap();
    assert_eq!(format!("{value}"), "#u8(2 3)");
}
