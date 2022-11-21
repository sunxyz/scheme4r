use crate::eval;
use crate::types::Type;
#[test]
fn test_add() {
    assert_eq!( Type::Numbers(6),eval("(+ 1 2 3)").unwrap(),"number not eq");
}
#[test]
fn test_subtract() {
    assert_eq!( Type::Numbers(-6),eval("(- 0 1 2 3)").unwrap(),"number not eq");
}
