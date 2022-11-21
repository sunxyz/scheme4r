use crate::eval;
use crate::types::Type;
#[test]
fn test_set() {
    assert_eq!(
        Type::Numbers(6),
        eval("((define a (+ 1 2 3)) (+ a a) (set! a 6) a)").unwrap(),
        "define number not eq"
    );
}
