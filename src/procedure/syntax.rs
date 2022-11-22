use std::{collections::HashMap, rc::Rc};

use crate::eval;

use super::*;
// use crate::{env::Env, eval};
// // syntax_rules 匹配到规则 然后套入变量 进行转换 得到一个表达式
// // 转换后执行
fn syntax_rules(args: &mut ApplyArgs) -> Type {
    let expr = args.expr().clone();
    Type::procedure_of_rc(
        "syntax-rules",
        Rc::new(move |x| -> Type {
            let literal = expr.car();
            let rules = expr.cdr();
            for rule in rules.data().iter(){
                match rule {
                    Type::Lists(r)=>{
                        let sr_pattern =  r.car();
                        let template = r.cdr().car();
                        let expr = x.expr();
                        let mut env = HashMap::new();
                        let is_render = match sr_pattern {
                            Type::Lists(sr_pattern)=>{
                                 is_match(expr, sr_pattern, &mut env, false)
                            },
                            _=>  false
                        };
                        if is_render {
                            if let Type::Lists(template) = template {
                                return render(& template, &env);
                            }
                        }
                    },
                    _=>return Type::Nil
                }
            }
            Type::Nil
        }),
    )
}
// (x ... (y)) (1 (2))
fn is_match(expr: &List, sr_pattern: List, e: &mut HashMap<Symbol, Type>, is_rev: bool) -> bool {
    let ellipsis_count = sr_pattern
        .data()
        .iter()
        .filter(|x| {
            if let Type::Symbols(v) = *x {
                v.as_str() == "..."
            } else {
                false
            }
        })
        .count();
    if expr.is_nil() && sr_pattern.is_nil() {
        return true;
    }
    if ellipsis_count > 1
        || sr_pattern.is_nil()
        || (ellipsis_count < 1 && expr.len() != sr_pattern.len())
    {
        return false;
    }
    let mut expr = expr;
    let mut sr_pattern = sr_pattern;
    // println!("expr {}", expr);
    // println!("sr_pattern {}", sr_pattern);
    let car = sr_pattern.car();
    let value = expr.car();
    match car {
        Type::Symbols(v) => {
            if &v == "..." {
                if sr_pattern.len() == 1 {
                    let data = if is_rev {
                        let mut d = expr.data();
                        d.reverse();
                        List::of(d)
                    } else {
                        expr.clone()
                    };
                    e.insert(v, Type::Lists(data));
                    return true;
                } else {
                    let mut t = expr.data();
                    t.reverse();
                    expr = &List::of(t);
                    let mut t = sr_pattern.data();
                    t.reverse();
                    sr_pattern = List::of(t);
                    return is_match(expr, sr_pattern, e, true);
                }
            } else if &v == "_" {
                return is_match(&mut expr.cdr(), sr_pattern.cdr(), e, is_rev);
            } else {
                e.insert(v, value);
                return is_match(&mut expr.cdr(), sr_pattern.cdr(), e, is_rev);
            }
        }
        Type::Lists(p) => {
            if let Type::Lists(v) = value {
                let r = is_match(&mut v.clone(), p, e, is_rev);
                if r {
                    return is_match(&mut expr.cdr(), sr_pattern.cdr(), e, is_rev);
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }
        _ => return false,
    };
}

fn render(template: &List, env: &HashMap<Symbol, Type>) -> Type {
    let mut r = List::new();
    for item in template.data().iter() {
        match item {
            Type::Symbols(s) => {
                let v = if env.contains_key(s) {
                    env.get(s).unwrap()
                } else {
                    item
                };
                r.push(v.clone())
            }
            Type::Lists(l) => r.push(render(l, env)),
            _ => r.push(item.clone()),
        }
    }
    Type::Lists(r)
}

#[test]
pub fn test_is_match() {
    let pattern = eval("’(x ... y)").unwrap();
    let args = eval("’(1 2 3 4 5)").unwrap();
    if let Type::Lists(pattern) = pattern {
        if let Type::Lists(args) = args {
            let mut e = HashMap::new();
            println!(
                "{} = {} ? => {}",
                pattern,
                args,
                is_match(&mut args, pattern.clone(), &mut e, false)
            );
            for ele in e {
                println!("{}=>{}", ele.0, ele.1);
            }
        }
    }
}

#[test]
pub fn test_is_match2() {
    let pattern = eval("’((x z _ _) ... y)").unwrap();
    let args = eval("’((1 3 6 9) 2 3 4 5)").unwrap();
    if let Type::Lists(pattern) = pattern {
        if let Type::Lists(args) = args {
            let mut e = HashMap::new();
            println!(
                "{} = {} ? => {}",
                pattern,
                args,
                is_match(&mut args, pattern.clone(), &mut e, false)
            );
            for ele in e {
                println!("{}=>{}", ele.0, ele.1);
            }
        }
    }
}

#[test]
pub fn test_render() {
    let pattern = eval("’(x ... y)").unwrap();
    let args = eval("’(1 2 3 4 5)").unwrap();
    let template = eval("’(if (> x y) (y) ...)").unwrap();
    if let Type::Lists(pattern) = pattern {
        if let Type::Lists(args) = args {
            let mut e = HashMap::new();
            println!(
                "{} = {} ? => {}",
                pattern,
                args,
                is_match(&mut args, pattern.clone(), &mut e, false)
            );
            for ele in e.clone() {
                println!("{}=>{}", ele.0, ele.1);
            }
            if let Type::Lists(template) = template {
                let v = render(&template, &e);
                println!("render {} => {}", template, v);
            }
        }
    }
}
