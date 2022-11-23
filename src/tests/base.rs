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
#[test]
fn test_quote(){
    assert_eq!(eval("'3").unwrap().to_string(),"3");
    assert_eq!(eval("(quote 3)").unwrap().to_string(),"3");
    assert_eq!(eval("(quote 3)").unwrap(), eval("'3").unwrap());
    assert_eq!(eval("'(+ 3 4)").unwrap(), eval("(quote (+ 3 4))").unwrap());
    // assert_eq!(eval("'\"hi\"").unwrap().to_string(),"hi");
    // assert_eq!(eval("'a").unwrap().to_string(),"a");
    let t = eval("'(1 2 (+ 3 4))").unwrap().to_string();
    println!("---{}",t);
    // assert_eq!(eval("'(+ 3 4)").unwrap().to_string(),"(list '+ '3 '4)");
}