use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::{env::RefEnv, eval};

use super::*;
// use crate::{env::Env, eval};
// // syntax_rules 匹配到规则 然后套入变量 进行转换 得到一个表达式
// // 转换后执行
fn syntax_rules(expr: SExpr) -> Type {
    Type::procedure_of_rc(
        "syntax-rules",
        Rc::new(move |x| -> Type {
            let literal = expr.car();
            let ignores = if let Type::Lists(ignore) = literal {
                ignore
                    .data()
                    .iter()
                    .filter(|x| {
                        if let Type::Symbols(_) = x {
                            true
                        } else {
                            false
                        }
                    })
                    .map(|t| {
                        if let Type::Symbols(v) = t {
                            v.clone()
                        } else {
                            "NONE".to_string()
                        }
                    })
                    .collect::<HashSet<Symbol>>()
            } else {
                HashSet::new()
            };
            let rules = expr.cdr();

            let expr = x.expr();
            // println!("rules:{}",rules);
            println!("expr:{}",expr);
            for rule in rules.data().iter() {
                match rule {
                    Type::Lists(r) => {
                        let sr_pattern = r.car();
                        let template = r.cdr().car();
                        // println!("sr_pattern：{}",sr_pattern);
                        let mut env = HashMap::new();
                        let is_render = match sr_pattern {
                            Type::Lists(sr_pattern) => is_match(expr, &sr_pattern, &mut env, false),
                            _ => false,
                        };
                        if is_render {
                            if let Type::Lists(template) = template {
                                return render(&template, &env, &ignores);
                            }
                        }
                    }
                    _ => return Type::Error(format!("check syntax-rules rules:{} ", rules)),
                }
            }
            Type::Error(format!(
                "check syntax-rules {} not pattern expr:{} ",
                rules, expr
            ))
        }),
    )
}


// fn define_syntax(args:&mut ApplyArgs) ->Type{
//     let name = args.expr().car();
//     let expr = args.expr().cdr();
//     let 
// }

fn define_syntax( a:&mut ApplyArgs) -> Type {
    let name = if let Symbols(n)= a.expr().car(){n}else{"".to_string()};
    if name.is_empty(){
        return Type::Error(format!("check define-syntax name :{}", a.expr().car()));
    }
    let rules = a.expr().cdr();
    let f = a.inter(&Type::Lists(rules));
    let proc = Type::procedure_of_rc(
        name.as_str(),
        Rc::new(move |args: &mut ApplyArgs| -> Type {
            if let Type::Procedures(p) = f.clone() {
            //    let d =  args.expr().data();
            //     let list = List::new();
            //     let mut args0 = args.clone_of()
                println!(";;;{}",args.expr());
                let v = p.call(&mut args0);
                return args0.inter(&v);
            }
            Nil
        }),
    );
    a.env().ref_write().define(name.as_str(), proc);
    Nil
}
// (x ... (y)) (1 (2))
fn is_match(expr: &List, sr_pattern: &List, e: &mut HashMap<Symbol, Type>, is_rev: bool) -> bool {
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
                    let mut expr_data = expr.data();
                    expr_data.reverse();
                    let mut sr_pattern_data = sr_pattern.data();
                    sr_pattern_data.reverse();
                    return is_match(&List::of(expr_data), &List::of(sr_pattern_data), e, true);
                }
            } else if &v == "_" {
                return is_match(&expr.cdr(), &sr_pattern.cdr(), e, is_rev);
            } else {
                e.insert(v, value);
                return is_match(&expr.cdr(), &sr_pattern.cdr(), e, is_rev);
            }
        }
        Type::Lists(p) => {
            if let Type::Lists(v) = value {
                let r = is_match(&v, &p, e, is_rev);
                if r {
                    return is_match(&expr.cdr(), &sr_pattern.cdr(), e, is_rev);
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

fn render(template: &List, env: &HashMap<Symbol, Type>, ignores: &HashSet<Symbol>) -> Type {
    let mut r = List::new();
    for item in template.data().iter() {
        match item {
            Type::Symbols(s) => {
                let v = if env.contains_key(s) && !ignores.contains(s) {
                    env.get(s).unwrap()
                } else {
                    item
                };
                r.push(v.clone())
            }
            Type::Lists(l) => r.push(render(l, env, ignores)),
            _ => r.push(item.clone()),
        }
    }
    Type::Lists(r)
}

pub fn reg_procedure(env: &mut Env) {
    env.reg_procedure("syntax-rules", |args| syntax_rules(args.expr().clone()));
    env.reg_procedure("define-syntax", define_syntax);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval;
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
                    is_match(&args, &pattern, &mut e, false)
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
                    is_match(&args, &pattern, &mut e, false)
                );
                for ele in e {
                    println!("{}=>{}", ele.0, ele.1);
                }
            }
        }
    }

    #[test]
    pub fn test_render() {
        let pattern = eval("’((x) ... y)").unwrap();
        let args = eval("’(1 2 3 4 5)").unwrap();
        let template = eval("’(if (> x y) (y) ...)").unwrap();
        if let Type::Lists(pattern) = pattern {
            if let Type::Lists(args) = args {
                let mut e = HashMap::new();
                println!(
                    "{} = {} ? => {}",
                    pattern,
                    args,
                    is_match(&args, &pattern, &mut e, false)
                );
                for ele in e.clone() {
                    println!("{}=>{}", ele.0, ele.1);
                }
                if let Type::Lists(template) = template {
                    let v = render(&template, &e, &HashSet::new());
                    println!("render {} => {}", template, v);
                }
            }
        }
    }
}
