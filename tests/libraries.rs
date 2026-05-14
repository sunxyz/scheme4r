use scheme4r::{eval, Value};

#[test]
fn define_library_and_import_work_for_exported_bindings() {
    let value = eval(
        "\
        (define-library (demo util)
          (export answer flag (rename internal-double double))
          (import (scheme base))
          (begin
            (define answer 42)
            (define flag #t)
            (define (internal-double x) (+ x x))))
        (import (demo util))
        (list answer (double 5) flag)
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(42 10 #t)");
}

#[test]
fn import_sets_support_only_except_prefix_and_rename() {
    let value = eval(
        "\
        (define-library (demo names)
          (export alpha beta gamma)
          (import (scheme base))
          (begin
            (define alpha 1)
            (define beta 2)
            (define gamma 3)))
        (import (only (demo names) alpha gamma))
        (import (prefix (only (demo names) beta) lib:))
        (import (rename (except (demo names) beta gamma) (alpha renamed-alpha)))
        (list alpha gamma lib:beta renamed-alpha)
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(1 3 2 1)");
}

#[test]
fn importing_scheme_base_is_allowed() {
    let value = eval(
        "\
        (import (scheme base))
        (+ 19 23)
        ",
    )
    .unwrap();

    assert!(matches!(value, Value::Number(42)));
}
