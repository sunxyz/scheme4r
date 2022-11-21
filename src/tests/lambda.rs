use crate::{eval, types::Type};

#[test]
fn test_lambda() {
    assert_eq!(Type::Numbers(6), eval("((lambda (x) (+ x x)) 3)").unwrap());
}

#[test]
fn test_lambda_name() {
    assert_eq!(
        Type::Numbers(6),
        eval("((define f (lambda (x) (+ x x))) (f 3))").unwrap()
    );
}

#[test]
fn test_lambda_produce() {
    assert_eq!(
        Type::Numbers(9),
        eval("((define f (lambda (op x) (op x x))) (f * 3))").unwrap()
    );
}

#[test]
fn test_lambda_2() {
    assert_eq!(
        Type::Numbers(65),
        eval("((lambda (x y) (+ x y)) 23 42)").unwrap()
    );
}

#[test]
#[should_panic]
fn test_lambda_3() {
    assert_eq!(
        Type::Numbers(36),
        eval("((lambda (x y) (+ x y)) 23 42)").unwrap()
    );
}
