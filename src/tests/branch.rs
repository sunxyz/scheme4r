use crate::{types::Type, eval};

#[test]
fn test_branch() {
    assert_eq!( Type::Booleans(true),eval("(if (- 3 2) #t #f)").unwrap());
}

#[test]
fn test(){
    println!("{}", eval("(#(0 (2 2 2 2) 'Anna'))").unwrap());
}

#[test]
fn test_demo() {
    println!("{}", eval("(+ 14 (* 23 42))").unwrap());
}