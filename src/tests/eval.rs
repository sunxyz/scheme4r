use crate::{eval};

#[test]
fn test_eval() {
    let r = format!("{}",eval("((syntax-rules()((_ x  y) (+ x y x)))test 1 2)").unwrap());
    assert_eq!(r,"(+ 1 2 1)");
    let r = eval(format!("((define n 5)(eval (+ {} n)))",r).as_str()).unwrap();
    assert_eq!(r.to_string(), "9");
}