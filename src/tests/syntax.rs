use crate::{eval};

#[test]
fn test_syntax_rules() {
    let r = format!("{}",eval("((syntax-rules()((_ x  y) (+ x y x)))test 1 2)").unwrap());
    assert_eq!(r,"(+ 1 2 1)");
}

#[test]
fn test_syntax_rules_apply() {
    let r = format!("{}",eval("(apply ((syntax-rules()((_ x  y) (+ x y x)))test 1 2))").unwrap());
    assert_eq!(r,"4");
}
#[test]
fn test_syntax_rules1(){
    let r = format!("{}",eval("((syntax-rules()((_ x ) (set! x nil)) ((_ x y) (set! x y)))test v)").unwrap());
    assert_eq!(r,"(set! v nil)");
}
#[test]
fn test_syntax_rules2(){
    let r = format!("{}",eval("((syntax-rules()((_ x ) (set! x nil)) ((_ x y) (set! x y)))test v 1)").unwrap());
    assert_eq!(r,"(set! v 1)");
}

