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
    let r = format!("{}",eval("((define r (syntax-rules()((_ x ) (set! x nil)) ((_ x y) (set! x y))))(r v v 1))").unwrap());
    // println!("--00 {}",eval("(apply '(test v 1))").unwrap());
    assert_eq!(r,"(set! v 1)");
}

#[test]
fn test_define_syntax_rules(){
    let r = format!("{}",eval("((define v 6)(define-syntax r (syntax-rules()((_ x ) (set! x nil)) ((_ x y) (set! x y))))(r v 1) v)").unwrap());
    // println!("--00 {}",eval("(apply '(test v 1))").unwrap());
    assert_eq!(r,"1");
}



#[test]
#[should_panic]
fn test_define_syntax_rules_2(){
    let r = format!("{}",eval("((define z 6)(define-syntax r (syntax-rules()((_ x ) (set! x nil)) ((_ x y) (set! x y))))(r v 1) v)").unwrap());
    // println!("--00 {}",eval("(apply '(test v 1))").unwrap());
    assert_eq!(r,"1");
}

#[test]
fn test_define_syntax_rules_3(){
    let r = format!("{}",eval("((define-syntax r (syntax-rules()((_ x ) (set! x nil)) ((_ x y) (set! x y))))(r v v 1 6 9))").unwrap());
    // println!("--00 {}",eval("(apply '(test v 1))").unwrap());
    assert_eq!(r,"<error:check syntax-rules (((_ x) (set! x nil)) ((_ x y) (set! x y))) not pattern expr:(r v v 1 6 9) >");
}


#[test]
fn test_define_syntax_rules_4(){
    let r = format!("{}",eval("((define v 0)(define-syntax r (syntax-rules()((_ x ) (set! x nil)) ((_ x y) (set! x y))))(r v 6) v)").unwrap());
    // println!("--00 {}",eval("(apply '(test v 1))").unwrap());
    assert_eq!(r,"6");
}

#[test]
fn test_define_syntax_rules_when(){
    let r = format!("{}",eval("((define-syntax when (syntax-rules() ((_ b p) (if b p))))(when (+ 1 2 3) 6))").unwrap());
    // println!("--00 {}",eval("(apply '(test v 1))").unwrap());
    assert_eq!(r,"6");
}

#[test]
fn test_define_syntax_rules_and(){
    let r = format!("{}",eval("((define-syntax and (syntax-rules() ((_ a)(a)) ((_ a b) (if a b a)) ((_ a b ...) (if a (and b ...) a))))(and #t #t #t #t))").unwrap());
    // println!("--00 {}",eval("(apply '(test v 1))").unwrap());
    assert_eq!(r,"#t");
}

#[test]
fn test_define_syntax_rules_and_1(){
    let r = format!("{}",eval("((define-syntax and (syntax-rules() ((_ a)(a)) ((_ a b) (if a b a)) ((_ a b ...) (if a (and b ...) a))))(and #t #t #t #f))").unwrap());
    assert_eq!(r,"#f");
    let r = format!("{}",eval("((define-syntax and (syntax-rules() ((_ a)(a)) ((_ a b) (if a b a)) ((_ a b ...) (if a (and b ...) a))))(and #t #f #t #t))").unwrap());
    assert_eq!(r,"#f");
    let r = format!("{}",eval("((define-syntax and (syntax-rules() ((_ a)(a)) ((_ a b) (if a b a)) ((_ a b ...) (if a (and b ...) a))))(and #t 0 #t #t))").unwrap());
    assert_eq!(r,"0");
    let r = format!("{}",eval("((define-syntax and (syntax-rules() ((_ a)(a)) ((_ a b) (if a b a)) ((_ a b ...) (if a (and b ...) a))))(and #t 1 #t (list 1 2 3)))").unwrap());
    assert_eq!(r,"(1 2 3)");
}

#[test]
fn test_define_syntax_rules_or(){
    let r = format!("{}",eval("((define-syntax or (syntax-rules() ((_ a)(a)) ((_ a b) (if a a b)) ((_ a b ...) (if a a (or b ...)))))(or #f #f #t #t))").unwrap());
    assert_eq!(r,"#t");
    let r = format!("{}",eval("((define-syntax or (syntax-rules() ((_ a)(a)) ((_ a b) (if a a b)) ((_ a b ...) (if a a (or b ...)))))(or #f #f #f))").unwrap());
    assert_eq!(r,"#f");
    let r = format!("{}",eval("((define-syntax or (syntax-rules() ((_ a)(a)) ((_ a b) (if a a b)) ((_ a b ...) (if a a (or b ...)))))(or #f (list 1 2 3) #f))").unwrap());
    assert_eq!(r,"(1 2 3)");
}


#[test]
fn test_define_syntax_rules_not(){
    let r = format!("{}",eval("((define-syntax not (syntax-rules() ((_ a)(if a #f #t))))(not #f))").unwrap());
    assert_eq!(r,"#t");
}