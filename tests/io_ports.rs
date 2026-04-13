use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use scheme4r::{eval, interpreter, Environment, Value};

#[test]
fn read_works_with_input_string_ports_and_eof() {
    let value = eval(
        "\
        (let ((p (open-input-string \"1 2\")))
          (list (read p) (read p) (eof-object? (read p))))
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(1 2 #t)");
}

#[test]
fn read_char_and_peek_char_share_input_cursor_with_read() {
    let value = eval(
        "\
        (let ((p (open-input-string \"a 12\")))
          (list (peek-char p)
                (read-char p)
                (peek-char p)
                (read p)
                (eof-object? (peek-char p))
                (eof-object? (read-char p))))
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(#\\a #\\a #\\space 12 #t #t)");
}

#[test]
fn read_line_reads_until_newline_and_then_eof() {
    let value = eval(
        "\
        (let ((p (open-input-string \"hello\\nworld\")))
          (list (read-line p)
                (read-line p)
                (eof-object? (read-line p))))
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(\"hello\" \"world\" #t)");
}

#[test]
fn output_string_ports_capture_display_write_and_newline() {
    let value = eval(
        "\
        (let ((p (open-output-string)))
          (display \"hi\" p)
          (write '(1 2) p)
          (write-char #\\! p)
          (newline p)
          (get-output-string p))
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::String(text) if text == "hi(1 2)!\n"));
}

#[test]
fn builtin_eval_uses_current_environment() {
    let value = eval(
        "\
        (define x 40)
        (list (eval '(+ x 2))
              (eval \"(+ 5 6)\"))
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(42 11)");
}

#[test]
fn load_reads_file_and_updates_environment() {
    let mut path = std::env::temp_dir();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("scheme4r-load-{unique}.scm"));

    fs::write(
        &path,
        "(define loaded 41)\n(set! loaded (+ loaded 1))\nloaded\n",
    )
    .unwrap();

    let env = Environment::standard();
    let path_string = path
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");

    let value = interpreter(&format!("(load \"{path_string}\")"), env.clone()).unwrap();
    assert!(matches!(value, Value::Number(42)));

    let value = interpreter("loaded", env).unwrap();
    assert!(matches!(value, Value::Number(42)));

    let _ = fs::remove_file(path);
}

#[test]
fn output_file_ports_write_to_files() {
    let mut path = std::env::temp_dir();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("scheme4r-output-{unique}.txt"));

    let path_string = path
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    eval(&format!(
        "\
        (let ((p (open-output-file \"{path_string}\")))
          (display \"hello\" p)
          (newline p)
          (write '(1 2) p)
          (close-output-port p))
        "
    ))
    .unwrap();

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "hello\n(1 2)");

    let _ = fs::remove_file(path);
}
