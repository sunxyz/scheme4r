use scheme4r::{eval, Value};

#[test]
fn member_and_assoc_variants_return_expected_matches() {
    let value = eval("(memq 'b '(a b c))").unwrap();
    assert_eq!(format!("{value}"), "(b c)");

    let value = eval("(memv 2 '(1 2 3))").unwrap();
    assert_eq!(format!("{value}"), "(2 3)");

    let value = eval("(member '(2) '((1) (2) (3)))").unwrap();
    assert_eq!(format!("{value}"), "((2) (3))");

    let value = eval("(member 'z '(a b c))").unwrap();
    assert!(matches!(value, Value::Boolean(false)));

    let value = eval("(assq 'b '((a . 1) (b . 2) (c . 3)))").unwrap();
    assert_eq!(format!("{value}"), "(b . 2)");

    let value = eval("(assoc '(b) '(((a) . 1) ((b) . 2)))").unwrap();
    assert_eq!(format!("{value}"), "((b) . 2)");
}

#[test]
fn list_copy_creates_a_distinct_spine() {
    let value = eval(
        "\
        (let ((xs (list 1 2 3)))
          (list (equal? xs (list-copy xs))
                (eq? xs (list-copy xs))
                (list-copy '(1 2 . 3))))
        ",
    )
    .unwrap();

    assert_eq!(format!("{value}"), "(#t #f (1 2 . 3))");
}

#[test]
fn make_list_and_list_star_work() {
    let value = eval("(make-list 3 'x)").unwrap();
    assert_eq!(format!("{value}"), "(x x x)");

    let value = eval("(list* 1 2 3 '(4 5))").unwrap();
    assert_eq!(format!("{value}"), "(1 2 3 4 5)");

    let value = eval("(list* 1 2 3)").unwrap();
    assert_eq!(format!("{value}"), "(1 2 . 3)");
}

#[test]
fn map_and_for_each_support_parallel_list_iteration() {
    let value = eval("(map (lambda (x) (* x x)) '(1 2 3))").unwrap();
    assert_eq!(format!("{value}"), "(1 4 9)");

    let value = eval("(map + '(1 2 3) '(10 20 30))").unwrap();
    assert_eq!(format!("{value}"), "(11 22 33)");

    let value = eval(
        "\
        (let ((port (open-output-string)))
          (for-each (lambda (x y) (display (+ x y) port))
                    '(1 2 3)
                    '(4 5 6))
          (get-output-string port))
        ",
    )
    .unwrap();
    assert_eq!(format!("{value}"), "\"579\"");
}

#[test]
fn pair_mutation_cxr_accessors_and_list_set_work() {
    let value = eval(
        "\
        (let ((pair (cons 1 2))
              (xs (list 'a 'b 'c))
              (nested '(root ((alpha beta) gamma delta) tail)))
          (set-car! pair 'left)
          (set-cdr! pair '(right))
          (list-set! xs 1 'updated)
          (list pair
                xs
                (caar '((1 2) (3 4)))
                (cadar '((1 2) (3 4)))
                (caddr '(10 20 30 40))
                (cadddr '(10 20 30 40 50))
                (caaadr nested)))
        ",
    )
    .unwrap();

    assert_eq!(
        format!("{value}"),
        "((left right) (a updated c) 1 2 30 40 alpha)"
    );
}
