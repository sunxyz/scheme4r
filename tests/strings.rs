use scheme4r::{eval, Value};

#[test]
fn string_construction_indexing_and_conversion_work() {
    let value = eval("(make-string 3 #\\x)").unwrap();
    assert_eq!(format!("{value}"), "\"xxx\"");

    let value = eval("(string-ref \"你好\" 1)").unwrap();
    assert!(matches!(value, Value::Character('好')));

    let value = eval("(substring \"你好世界\" 1 3)").unwrap();
    assert_eq!(format!("{value}"), "\"好世\"");

    let value = eval("(string->list \"a中b\")").unwrap();
    assert_eq!(format!("{value}"), "(#\\a #\\中 #\\b)");

    let value = eval("(list->string '(#\\a #\\中 #\\b))").unwrap();
    assert_eq!(format!("{value}"), "\"a中b\"");

    let value = eval("(string-copy \"你好世界\" 1 3)").unwrap();
    assert_eq!(format!("{value}"), "\"好世\"");
}

#[test]
fn string_and_char_comparisons_work() {
    let value = eval("(string<? \"abc\" \"abd\" \"zzz\")").unwrap();
    assert!(matches!(value, Value::Boolean(true)));

    let value = eval("(string>=? \"bbb\" \"bbb\" \"aaa\")").unwrap();
    assert!(matches!(value, Value::Boolean(true)));

    let value = eval("(char<? #\\a #\\b #\\c)").unwrap();
    assert!(matches!(value, Value::Boolean(true)));

    let value = eval("(char=? #\\中 #\\中)").unwrap();
    assert!(matches!(value, Value::Boolean(true)));

    let value = eval("(string-ci=? \"Hello\" \"hELLo\")").unwrap();
    assert!(matches!(value, Value::Boolean(true)));

    let value = eval("(string-ci<? \"abc\" \"BCD\")").unwrap();
    assert!(matches!(value, Value::Boolean(true)));

    let value = eval("(char-ci=? #\\A #\\a)").unwrap();
    assert!(matches!(value, Value::Boolean(true)));

    let value = eval("(char-ci<? #\\a #\\B)").unwrap();
    assert!(matches!(value, Value::Boolean(true)));
}

#[test]
fn char_classification_and_case_mapping_work() {
    let value = eval("(char-alphabetic? #\\A)").unwrap();
    assert!(matches!(value, Value::Boolean(true)));

    let value = eval("(char-numeric? #\\9)").unwrap();
    assert!(matches!(value, Value::Boolean(true)));

    let value = eval("(char-whitespace? #\\space)").unwrap();
    assert!(matches!(value, Value::Boolean(true)));

    let value = eval("(char-upper-case? #\\A)").unwrap();
    assert!(matches!(value, Value::Boolean(true)));

    let value = eval("(char-lower-case? #\\z)").unwrap();
    assert!(matches!(value, Value::Boolean(true)));

    let value = eval("(char-upcase #\\a)").unwrap();
    assert!(matches!(value, Value::Character('A')));

    let value = eval("(char-downcase #\\Z)").unwrap();
    assert!(matches!(value, Value::Character('z')));

    let value = eval("(char-foldcase #\\Z)").unwrap();
    assert!(matches!(value, Value::Character('z')));

    let value = eval("(digit-value #\\9)").unwrap();
    assert!(matches!(value, Value::Number(9)));

    let value = eval("(digit-value #\\A)").unwrap();
    assert!(matches!(value, Value::Boolean(false)));
}

#[test]
fn string_map_and_string_for_each_work() {
    let value = eval("(string-map char-upcase \"ab中\")").unwrap();
    assert!(matches!(value, Value::String(text) if text == "AB中"));

    let value = eval("(string-map (lambda (a b) (if (char<? a b) a b)) \"bd\" \"ac\")").unwrap();
    assert!(matches!(value, Value::String(text) if text == "ac"));

    let value = eval(
        "\
        (let ((p (open-output-string)))
          (string-for-each (lambda (ch) (write-char ch p)) \"Hi!\")
          (get-output-string p))
        ",
    )
    .unwrap();
    assert!(matches!(value, Value::String(text) if text == "Hi!"));
}

#[test]
fn mutable_strings_case_mapping_and_vector_conversion_work() {
    let value = eval(
        "\
        (let* ((s (string #\\a #\\b #\\c))
               (alias s))
          (string-set! s 1 #\\Z)
          (string-copy! s 0 \"xy\")
          (string-fill! s #\\! 2 3)
          (list s
                alias
                (string->vector s)
                (string->vector \"abcd\" 1 3)
                (vector->string #(#\\H #\\i #\\!) 0 2)
                (string-upcase \"ab中\")
                (string-downcase \"Ab中\")
                (string-foldcase \"Ab中\")))
        ",
    )
    .unwrap();

    assert_eq!(
        format!("{value}"),
        "(\"xy!\" \"xy!\" #(#\\x #\\y #\\!) #(#\\b #\\c) \"Hi\" \"AB中\" \"ab中\" \"ab中\")"
    );
}
