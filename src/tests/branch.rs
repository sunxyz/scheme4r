use crate::{types::Type, eval};

#[test]
fn test_boolean() {
    assert_eq!( Type::Booleans(true),eval("(if (- 3 2) #t #f)").unwrap());
}
