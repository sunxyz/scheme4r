use super::types::Type;
static VEC_PREFIX: &'static str = "#(";
static U8VEC_PREFIX: &'static str = "#u8(";
static PREFIX: &'static str = "(";
static SUFFIX: &'static str = ")";

pub fn parser(expr: String) -> Result<Type, String> {
    let e = parse0(expr.replace("'", "\""));
    println!("parser:{}", e);
    Ok(e)
}

fn parse0(expr: String) -> Type {
    if expr.starts_with(PREFIX) {
        parse_expr(expr)
    } else {
        parse_atom(&expr).unwrap()
    }
}

fn parse_expr(expr: String) -> Type {
    let mut stack = Vec::new();
    let mut next = true;
    let mut exp = expr.as_str();
    while next {
        exp = exp.trim();
        let is_vec = exp.starts_with(VEC_PREFIX);
        let is_u8_vec = exp.starts_with(U8VEC_PREFIX);
        let is_push = exp.starts_with(PREFIX) || is_vec || is_u8_vec;
        let index = if is_u8_vec {
            4
        } else if is_vec {
            2
        } else {
            1
        };
        let next_exp = &exp[index..];
        let to_index = get_to_index(next_exp);
        next = next_exp.find(SUFFIX) != None;
        let sub_exp = &next_exp[..to_index];
        // print!("sub_exp:{} next_exp:{} is_push:{} to_index:{}" , sub_exp, next_exp, is_push, to_index);
        if is_push {
            if is_u8_vec {
                let v = parse_vec(sub_exp)
                    .iter()
                    .map(|x| as_u8(x))
                    .collect::<Vec<u8>>();
                stack.push(Type::u8vector_of(v));
            } else if is_vec {
                stack.push(Type::vector_of(parse_vec(sub_exp)));
            } else {
                stack.push(Type::expr_of(parse_vec(sub_exp)));
            }
        } else {
            let brother = stack.pop().unwrap();
            if stack.is_empty() {
                stack.push(brother);
            } else {
                let mut parent = stack.pop().unwrap();
                match &mut parent {
                    Type::Lists(p) => {
                        p.push(brother);
                        p.push_vec(parse_vec(sub_exp));
                    }
                    Type::Vectors(p) => {
                        p.push(brother);
                        p.push_vec(parse_vec(sub_exp));
                    }
                    Type::ByteVectors(p) => {
                        p.push(as_u8(&brother));
                        let v = parse_vec(sub_exp)
                            .iter()
                            .map(|x| as_u8(x))
                            .collect::<Vec<u8>>();
                        p.push_vec(v);
                    }
                    _ => {}
                }
                stack.push(parent);
            }
        }

        // println!("stack: {}", stack.len());
        // println!("-----");
        // print!("old-exp:{}to_index:{}",exp, to_index);
        exp = exp[to_index + index..].trim();
        // println!("exp:{}",exp)
    }
    stack.pop().unwrap()
}

fn get_to_index(next_exp: &str) -> usize {
    // println!("________________________________________________________________{}", next_exp);
    let pre1 = next_exp.find(PREFIX);
    let pre0 = next_exp.find(VEC_PREFIX);
    let pre2 = next_exp.find(U8VEC_PREFIX);
    // min
    let mut t = vec![pre0, pre1, pre2]
        .iter()
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
        .collect::<Vec<usize>>();
    t.sort();
    let pre = t.get(0);
    let suf = next_exp.find(SUFFIX);
    if suf != None {
        let suf_index = suf.unwrap();
        if pre != None {
            let pre_index = pre.unwrap();
            if *pre_index < suf_index {
                *pre_index
            } else {
                suf_index
            }
        } else {
            suf_index
        }
    } else {
        0
    }
}

fn parse_vec(exp: &str) -> Vec<Type> {
    rep_str(exp.to_string())
        .trim()
        .split_whitespace()
        .map(|s| parse_atom(s).unwrap())
        .collect()
}

fn rep_str(str: String) -> String {
    if let Some(i) = str.find("'") {
        let sub = &str[i + 1..];
        if let Some(j) = sub.find("'") {
            let s = &sub[..j].replace(" ", "\\u0009");
            let mut all_str = String::new();
            let right_str = &str[..i + 1];
            let left_str = &sub[j..];
            all_str.push_str(right_str);
            all_str.push_str(s);
            all_str.push_str(rep_str(left_str.to_string()).as_str());
            return all_str;
        }
        return str;
    } else {
        return str;
    }
}

fn parse_atom(s: &str) -> Result<Type, String> {
    let t: Type = match s {
        "nil" => Type::Nil,
        "#t" => Type::Booleans(true),
        "#f" => Type::Booleans(false),
        _ => {
            if s.starts_with("’") {
                let v = s.replace("’", "");
                let r = parse0(v.clone());
                Type::expr_of(vec![Type::symbol_of(String::from("quote")), r])
            } else if s.starts_with("\"") && s.ends_with("\"") && s.len() > 2 {
                Type::string_of(
                    s[1..s.len() - 1]
                        .replace("\\u0009", " ")
                        .replace("\\r", "\r")
                        .replace("\\n", "\n")
                        .to_string(),
                )
            } else if s.starts_with("#\\") && s.len() == 2 {
                Type::character_of(s.chars().nth(2).unwrap())
            } else if s.starts_with(U8VEC_PREFIX) && s.len() > 4 {
                let v = parse0(s.replace(U8VEC_PREFIX, "("));
                let r = match v {
                    Type::Lists(v) => v.data(),
                    _ => vec![v],
                };
                Type::u8vector_of(r.iter().map(|x| as_u8(x)).collect::<Vec<u8>>())
            } else if s.starts_with(VEC_PREFIX) && s.len() > 2 {
                let v = parse0(s.replace(VEC_PREFIX, "("));
                let r = match v {
                    Type::Lists(v) => v.data(),
                    _ => vec![v],
                };
                Type::vector_of(r)
            } else if s.parse::<isize>().is_ok() {
                Type::integer_of(s.parse::<isize>().unwrap())
            } else if s.parse::<f64>().is_ok() {
                Type::float_of(s.parse::<f64>().unwrap())
            } else if s.starts_with(",@") {
                if s.len() > 2 {
                    Type::expr_of(vec![
                        Type::symbol_of(",@".to_string()),
                        Type::symbol_of(s[2..].to_string()),
                    ])
                } else {
                    Type::symbol_of(s.to_string())
                }
            } else {
                peel_onions(s, vec![",", "'"])
            }
        }
    };
    Ok(t)
}

fn peel_onions(s: &str, keys: Vec<&str>) -> Type {
    for key in keys {
        if s.starts_with(key) && s.len() > key.len() {
            return Type::expr_of(vec![
                Type::Symbols(key.to_string()),
                Type::Symbols(s[key.len()..].to_string()),
            ]);
        }
    }
    Type::Symbols(s.to_string())
}

fn as_u8(v: &Type) -> u8 {
    match v {
        Type::Numbers(v) => *v as u8,
        Type::Characters(v) => *v as u8,
        _ => 0,
    }
}
