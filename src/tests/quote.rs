use crate::eval;
use crate::types::Type;
#[test]
fn test_quote() {
    assert_eq!(
        Type::Symbols("a".to_string()),
        eval("(quote a)").unwrap(),
        "define quote not eq"
    );
}
#[test]
fn test_quote2() {
    assert_eq!(
        Type::Symbols("a".to_string()),
        eval("’a").unwrap(),
        "define quote not eq"
    );
}
#[test]
fn test_quote_expr() {
    assert_eq!(
        Type::expr_of(vec![
            Type::symbol_of("+".to_string()),
            Type::Numbers(1),
            Type::Numbers(2)
        ]),
        eval("(quote (+ 1 2))").unwrap(),
        "define quote not eq"
    );
}
// 后期修复
#[test]
fn test_quote_vec1() {
    assert_eq!(
        Type::vector_of(vec![
            Type::symbol_of("a".to_string()),
            Type::symbol_of("b".to_string()),
            Type::symbol_of("c".to_string())
        ]),
        eval("(quote #( a b c ))").unwrap(),
        "define quote not eq"
    );
}

#[test]
fn test_quote_vec2() {
    assert_eq!(
        Type::vector_of(vec![
            Type::symbol_of("a".to_string()),
            Type::symbol_of("b".to_string()),
            Type::symbol_of("c".to_string())
        ]),
        eval("’#( a b c )").unwrap(),
        "define quote not eq"
    );
}
