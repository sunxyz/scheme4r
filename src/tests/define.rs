use crate::eval;
use crate::types::Type;
#[test]
fn test_define() {
    assert_eq!(
        Type::Numbers(12),
        eval("((define a (+ 1 2 3)) (+ a a))").unwrap(),
        "define number not eq"
    );
}
