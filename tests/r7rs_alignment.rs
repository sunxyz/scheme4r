use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use scheme4r::{eval, ErrorKind, Value};

#[test]
fn case_lambda_dispatches_on_arity() {
    let value = eval(
        "\
        (define chooser
          (case-lambda
            (() 'zero)
            ((x) x)
            ((x y . rest) (list x y rest))))
        (list (chooser) (chooser 42) (chooser 1 2 3 4))
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(zero 42 (1 2 (3 4)))");
}

#[test]
fn define_values_and_let_values_bind_multiple_values() {
    let value = eval(
        "\
        (define-values (a b) (values 10 20))
        (let-values (((x y) (values 1 2))
                     ((z) (values 3)))
          (let*-values (((sum) (values (+ x y z)))
                        ((left right) (values a b)))
            (list sum left right)))
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(6 10 20)");
}

#[test]
fn eval_accepts_environment_specifier() {
    let value = eval("(eval '(+ 19 23) (environment '(scheme base)))").unwrap();
    assert!(matches!(value, Value::Number(42)));
}

#[test]
fn environment_specifier_uses_import_sets_instead_of_current_env() {
    let err = eval(
        "\
        (define hidden 41)
        (eval 'hidden (environment '(scheme base)))
        ",
    )
    .unwrap_err();

    assert_eq!(err.kind, ErrorKind::Name);
}

#[test]
fn scheme_base_environment_does_not_expose_write_bindings() {
    let err = eval("(eval 'display (environment '(scheme base)))").unwrap_err();
    assert_eq!(err.kind, ErrorKind::Name);

    let value = eval(
        "\
        (procedure? (eval 'display (environment '(scheme write))))
        ",
    )
    .unwrap();
    assert!(matches!(value, Value::Boolean(true)));
}

#[test]
fn parameterize_temporarily_overrides_parameter_values() {
    let value = eval(
        "\
        (define current-x (make-parameter 1))
        (list
          (current-x)
          (parameterize ((current-x 42))
            (current-x))
          (current-x))
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(1 42 1)");
}

#[test]
fn delay_and_force_memoize_results() {
    let value = eval(
        "\
        (define n 0)
        (define p (delay (begin (set! n (+ n 1)) n)))
        (list (force p) (force p) n)
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(1 1 1)");
}

#[test]
fn delay_force_flattens_nested_promises() {
    let value = eval(
        "\
        (define p (delay-force (delay 42)))
        (force p)
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Number(42)));
}

#[test]
fn guard_handles_raised_objects_and_parameterize_restores_on_error() {
    let value = eval(
        "\
        (define current-x (make-parameter 1))
        (guard (ex
                 ((equal? ex 'boom) (list 'handled (current-x)))
                 (else 'unexpected))
          (parameterize ((current-x 9))
            (raise 'boom)))
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(handled 1)");
}

#[test]
fn cond_expand_selects_matching_feature_branch() {
    let value = eval(
        "\
        (cond-expand
          ((library (scheme write)) 42)
          (else 0))
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Number(42)));
}

#[test]
fn include_and_include_ci_evaluate_file_forms() {
    let mut path = std::env::temp_dir();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("scheme4r-include-{unique}.scm"));

    fs::write(&path, "(define included 41)\n(set! included (+ included 1))\nincluded\n").unwrap();
    let path_string = path
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");

    let value = eval(&format!(
        "\
        (include \"{path_string}\")
        included
        "
    ))
    .unwrap();
    assert!(matches!(value, Value::Number(42)));

    fs::write(&path, "(DEFINE folded 41)\n(+ folded 1)\n").unwrap();
    let value = eval(&format!("(include-ci \"{path_string}\")")).unwrap();
    assert!(matches!(value, Value::Number(42)));

    let _ = fs::remove_file(path);
}

#[test]
fn load_resolves_relative_include_paths() {
    let mut dir = std::env::temp_dir();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    dir.push(format!("scheme4r-load-dir-{unique}"));
    fs::create_dir_all(&dir).unwrap();

    let child = dir.join("child.scm");
    let main = dir.join("main.scm");
    fs::write(&child, "(define relative-answer 42)\n").unwrap();
    fs::write(&main, "(include \"child.scm\")\nrelative-answer\n").unwrap();

    let main_string = main
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    let value = eval(&format!("(load \"{main_string}\")")).unwrap();
    assert!(matches!(value, Value::Number(42)));

    let _ = fs::remove_file(child);
    let _ = fs::remove_file(main);
    let _ = fs::remove_dir(dir);
}

#[test]
fn define_library_supports_cond_expand_and_include_library_declarations() {
    let mut path = std::env::temp_dir();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    path.push(format!("scheme4r-library-decls-{unique}.scm"));
    fs::write(
        &path,
        "(export answer)\n(import (scheme base))\n(begin (define answer 42))\n",
    )
    .unwrap();
    let path_string = path
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");

    let value = eval(&format!(
        "\
        (define-library (demo cond)
          (cond-expand
            ((library (scheme write))
             (include-library-declarations \"{path_string}\"))
            (else
             (export answer)
             (import (scheme base))
             (begin (define answer 0)))))
        (import (demo cond))
        answer
        "
    ))
    .unwrap();

    assert!(matches!(value, Value::Number(42)));
    let _ = fs::remove_file(path);
}

#[test]
fn define_library_can_export_syntax_bindings() {
    let value = eval(
        "\
        (define-library (demo syntax)
          (export when1)
          (import (scheme base))
          (begin
            (define-syntax when1
              (syntax-rules ()
                ((_ test body ...)
                 (if test (begin body ...)))))))
        (import (demo syntax))
        (define x 0)
        (when1 #t
          (set! x 42))
        x
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Number(42)));
}
