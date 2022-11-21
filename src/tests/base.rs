use crate::{types::Type, eval};

#[test]
fn test_boolean() {
    assert_eq!( Type::Booleans(true),eval("(#t)").unwrap());
}

#[test]
fn test_number() {
    assert_eq!( Type::Numbers(3),eval("(3)").unwrap());
    assert_eq!( Type::Numbers(65),eval("(+ 23 42)").unwrap());
    assert_eq!( Type::Numbers(980),eval("(+ 14 (* 23 42))").unwrap());
}