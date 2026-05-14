use super::*;

pub(super) fn install(env: &mut Environment) {
    define_builtin(env, "null?", is_null);
    define_builtin(env, "pair?", is_pair);
    define_builtin(env, "list?", is_list);
    define_builtin(env, "cons", cons);
    define_builtin(env, "car", car);
    define_builtin(env, "cdr", cdr);
    define_builtin(env, "set-car!", set_car);
    define_builtin(env, "set-cdr!", set_cdr);
    define_builtin(env, "caar", caar);
    define_builtin(env, "cadr", cadr);
    define_builtin(env, "cdar", cdar);
    define_builtin(env, "cddr", cddr);
    define_builtin(env, "caaar", caaar);
    define_builtin(env, "caadr", caadr);
    define_builtin(env, "cadar", cadar);
    define_builtin(env, "caddr", caddr);
    define_builtin(env, "cdaar", cdaar);
    define_builtin(env, "cdadr", cdadr);
    define_builtin(env, "cddar", cddar);
    define_builtin(env, "cdddr", cdddr);
    define_builtin(env, "caaaar", caaaar);
    define_builtin(env, "caaadr", caaadr);
    define_builtin(env, "caadar", caadar);
    define_builtin(env, "caaddr", caaddr);
    define_builtin(env, "cadaar", cadaar);
    define_builtin(env, "cadadr", cadadr);
    define_builtin(env, "caddar", caddar);
    define_builtin(env, "cadddr", cadddr);
    define_builtin(env, "cdaaar", cdaaar);
    define_builtin(env, "cdaadr", cdaadr);
    define_builtin(env, "cdadar", cdadar);
    define_builtin(env, "cdaddr", cdaddr);
    define_builtin(env, "cddaar", cddaar);
    define_builtin(env, "cddadr", cddadr);
    define_builtin(env, "cdddar", cdddar);
    define_builtin(env, "cddddr", cddddr);
    define_builtin(env, "make-list", make_list);
    define_builtin(env, "list", list);
    define_builtin(env, "list*", list_star);
    define_builtin(env, "length", length);
    define_builtin(env, "append", append);
    define_builtin(env, "reverse", reverse);
    define_builtin(env, "list-ref", list_ref);
    define_builtin(env, "list-tail", list_tail);
    define_builtin(env, "list-set!", list_set);
    define_builtin(env, "list-copy", list_copy);
    define_builtin(env, "memq", memq);
    define_builtin(env, "memv", memv);
    define_builtin(env, "member", member);
    define_builtin(env, "assq", assq);
    define_builtin(env, "assv", assv);
    define_builtin(env, "assoc", assoc);
    define_builtin(env, "map", map);
    define_builtin(env, "for-each", for_each);
}

macro_rules! define_cxr {
    ($func:ident, $name:literal, $path:literal) => {
        fn $func(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
            expect_arity($name, args, 1)?;
            traverse_cxr($name, &args[0], $path)
        }
    };
}

define_cxr!(caar, "caar", "aa");
define_cxr!(cadr, "cadr", "ad");
define_cxr!(cdar, "cdar", "da");
define_cxr!(cddr, "cddr", "dd");
define_cxr!(caaar, "caaar", "aaa");
define_cxr!(caadr, "caadr", "aad");
define_cxr!(cadar, "cadar", "ada");
define_cxr!(caddr, "caddr", "add");
define_cxr!(cdaar, "cdaar", "daa");
define_cxr!(cdadr, "cdadr", "dad");
define_cxr!(cddar, "cddar", "dda");
define_cxr!(cdddr, "cdddr", "ddd");
define_cxr!(caaaar, "caaaar", "aaaa");
define_cxr!(caaadr, "caaadr", "aaad");
define_cxr!(caadar, "caadar", "aada");
define_cxr!(caaddr, "caaddr", "aadd");
define_cxr!(cadaar, "cadaar", "adaa");
define_cxr!(cadadr, "cadadr", "adad");
define_cxr!(caddar, "caddar", "adda");
define_cxr!(cadddr, "cadddr", "addd");
define_cxr!(cdaaar, "cdaaar", "daaa");
define_cxr!(cdaadr, "cdaadr", "daad");
define_cxr!(cdadar, "cdadar", "dada");
define_cxr!(cdaddr, "cdaddr", "dadd");
define_cxr!(cddaar, "cddaar", "ddaa");
define_cxr!(cddadr, "cddadr", "ddad");
define_cxr!(cdddar, "cdddar", "ddda");
define_cxr!(cddddr, "cddddr", "dddd");

fn is_null(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("null?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::EmptyList)))
}

fn is_pair(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("pair?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::Pair(_))))
}

fn is_list(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("list?", args, 1)?;
    Ok(Value::Boolean(args[0].is_proper_list()))
}

fn cons(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("cons", args, 2)?;
    Ok(Value::pair(args[0].clone(), args[1].clone()))
}

fn car(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("car", args, 1)?;
    let pair = expect_pair("car", &args[0])?;
    let value = pair.borrow().car.clone();
    Ok(value)
}

fn cdr(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("cdr", args, 1)?;
    let pair = expect_pair("cdr", &args[0])?;
    let value = pair.borrow().cdr.clone();
    Ok(value)
}

fn set_car(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("set-car!", args, 2)?;
    let pair = expect_pair("set-car!", &args[0])?;
    pair.borrow_mut().car = args[1].clone();
    Ok(Value::Unspecified)
}

fn set_cdr(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("set-cdr!", args, 2)?;
    let pair = expect_pair("set-cdr!", &args[0])?;
    pair.borrow_mut().cdr = args[1].clone();
    Ok(Value::Unspecified)
}

fn make_list(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 2 {
        return Err(SchemeError::arity("'make-list' expects 1 or 2 arguments"));
    }
    let len = expect_index("make-list", &args[0])?;
    let fill = args.get(1).cloned().unwrap_or(Value::Unspecified);
    Ok(Value::list(vec![fill; len]))
}

fn list(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::list(args.to_vec()))
}

fn list_star(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    let (tail, items) = args
        .split_last()
        .ok_or_else(|| SchemeError::arity("'list*' expects at least 1 argument"))?;
    Ok(Value::list_with_tail(items.to_vec(), tail.clone()))
}

fn length(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("length", args, 1)?;
    let items = expect_list("length", &args[0])?;
    Ok(Value::Number(items.len() as i64))
}

fn append(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() {
        return Ok(Value::EmptyList);
    }

    let mut items = Vec::new();
    for value in &args[..args.len() - 1] {
        items.extend(expect_list("append", value)?);
    }

    Ok(Value::list_with_tail(
        items,
        args.last().cloned().unwrap_or(Value::EmptyList),
    ))
}

fn reverse(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("reverse", args, 1)?;
    let mut items = expect_list("reverse", &args[0])?;
    items.reverse();
    Ok(Value::list(items))
}

fn list_ref(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("list-ref", args, 2)?;
    let items = expect_list("list-ref", &args[0])?;
    let index = expect_index("list-ref", &args[1])?;
    items
        .get(index)
        .cloned()
        .ok_or_else(|| SchemeError::runtime("'list-ref' index out of range"))
}

fn list_tail(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("list-tail", args, 2)?;
    let index = expect_index("list-tail", &args[1])?;
    let mut current = args[0].clone();
    for _ in 0..index {
        match current {
            Value::Pair(pair) => {
                current = pair.borrow().cdr.clone();
            }
            Value::EmptyList => {
                return Err(SchemeError::runtime("'list-tail' index out of range"));
            }
            _ => {
                return Err(SchemeError::type_error(
                    "'list-tail' expected a proper list",
                ));
            }
        }
    }
    Ok(current)
}

fn list_set(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("list-set!", args, 3)?;
    let index = expect_index("list-set!", &args[1])?;
    let pair = nth_pair("list-set!", &args[0], index)?;
    pair.borrow_mut().car = args[2].clone();
    Ok(Value::Unspecified)
}

fn list_copy(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("list-copy", args, 1)?;
    copy_pair_chain("list-copy", &args[0])
}

fn memq(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("memq", args, 2)?;
    find_member_tail("memq", &args[0], &args[1], Value::eqv)
}

fn memv(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("memv", args, 2)?;
    find_member_tail("memv", &args[0], &args[1], Value::eqv)
}

fn member(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("member", args, 2)?;
    find_member_tail("member", &args[0], &args[1], Value::equal)
}

fn assq(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("assq", args, 2)?;
    find_assoc_entry("assq", &args[0], &args[1], Value::eqv)
}

fn assv(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("assv", args, 2)?;
    find_assoc_entry("assv", &args[0], &args[1], Value::eqv)
}

fn assoc(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("assoc", args, 2)?;
    find_assoc_entry("assoc", &args[0], &args[1], Value::equal)
}

fn map(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() < 2 {
        return Err(SchemeError::arity(
            "'map' expects a procedure and at least one list",
        ));
    }

    let procedure = args[0].clone();
    let mut cursors = args[1..].to_vec();
    let mut results = Vec::new();

    while let Some((call_args, next_cursors)) = next_parallel_list_frame("map", &cursors)? {
        let value = engine.apply(procedure.clone(), engine.current_env(), call_args)?;
        results.push(value);
        cursors = next_cursors;
    }

    Ok(Value::list(results))
}

fn for_each(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() < 2 {
        return Err(SchemeError::arity(
            "'for-each' expects a procedure and at least one list",
        ));
    }

    let procedure = args[0].clone();
    let mut cursors = args[1..].to_vec();

    while let Some((call_args, next_cursors)) = next_parallel_list_frame("for-each", &cursors)? {
        engine.apply(procedure.clone(), engine.current_env(), call_args)?;
        cursors = next_cursors;
    }

    Ok(Value::Unspecified)
}

fn copy_pair_chain(name: &str, value: &Value) -> Result<Value, SchemeError> {
    match value {
        Value::EmptyList => Ok(Value::EmptyList),
        Value::Pair(pair_ref) => {
            let (car, cdr) = {
                let pair = pair_ref.borrow();
                (pair.car.clone(), pair.cdr.clone())
            };
            Ok(Value::pair(car, copy_pair_chain(name, &cdr)?))
        }
        other => {
            let _ = name;
            Ok(other.clone())
        }
    }
}

fn find_member_tail<F>(
    name: &str,
    needle: &Value,
    list: &Value,
    cmp: F,
) -> Result<Value, SchemeError>
where
    F: Fn(&Value, &Value) -> bool,
{
    let mut current = list.clone();

    loop {
        match current {
            Value::Pair(pair_ref) => {
                let (car, cdr) = {
                    let pair = pair_ref.borrow();
                    (pair.car.clone(), pair.cdr.clone())
                };
                if cmp(needle, &car) {
                    return Ok(Value::Pair(pair_ref));
                }
                current = cdr;
            }
            Value::EmptyList => return Ok(Value::Boolean(false)),
            other => {
                return Err(SchemeError::type_error(format!(
                    "'{name}' expected a proper list, got {other}"
                )));
            }
        }
    }
}

fn find_assoc_entry<F>(
    name: &str,
    needle: &Value,
    list: &Value,
    cmp: F,
) -> Result<Value, SchemeError>
where
    F: Fn(&Value, &Value) -> bool,
{
    let mut current = list.clone();

    loop {
        match current {
            Value::Pair(pair_ref) => {
                let (entry, cdr) = {
                    let pair = pair_ref.borrow();
                    (pair.car.clone(), pair.cdr.clone())
                };
                let entry_pair = match &entry {
                    Value::Pair(entry_pair) => entry_pair.clone(),
                    other => {
                        return Err(SchemeError::type_error(format!(
                            "'{name}' expected an association list, got entry {other}"
                        )));
                    }
                };
                let key = {
                    let entry = entry_pair.borrow();
                    entry.car.clone()
                };
                if cmp(needle, &key) {
                    return Ok(entry);
                }
                current = cdr;
            }
            Value::EmptyList => return Ok(Value::Boolean(false)),
            other => {
                return Err(SchemeError::type_error(format!(
                    "'{name}' expected a proper list, got {other}"
                )));
            }
        }
    }
}

fn next_parallel_list_frame(
    name: &str,
    cursors: &[Value],
) -> Result<Option<(Vec<Value>, Vec<Value>)>, SchemeError> {
    let mut args = Vec::with_capacity(cursors.len());
    let mut next = Vec::with_capacity(cursors.len());
    let mut saw_pair = false;
    let mut saw_empty = false;

    for cursor in cursors {
        match cursor {
            Value::Pair(pair_ref) => {
                let (car, cdr) = {
                    let pair = pair_ref.borrow();
                    (pair.car.clone(), pair.cdr.clone())
                };
                saw_pair = true;
                args.push(car);
                next.push(cdr);
            }
            Value::EmptyList => {
                saw_empty = true;
            }
            other => {
                return Err(SchemeError::type_error(format!(
                    "'{name}' expected proper lists, got {other}"
                )));
            }
        }
    }

    if saw_empty {
        if saw_pair {
            return Err(SchemeError::runtime(format!(
                "'{name}' expected lists of equal length"
            )));
        }
        return Ok(None);
    }

    Ok(Some((args, next)))
}

fn traverse_cxr(name: &str, value: &Value, path: &str) -> Result<Value, SchemeError> {
    let mut current = value.clone();
    for op in path.chars().rev() {
        let pair = expect_pair(name, &current)?;
        current = match op {
            'a' => pair.borrow().car.clone(),
            'd' => pair.borrow().cdr.clone(),
            _ => unreachable!(),
        };
    }
    Ok(current)
}

fn nth_pair(name: &str, value: &Value, index: usize) -> Result<Rc<RefCell<PairCell>>, SchemeError> {
    let mut current = value.clone();
    for _ in 0..index {
        current = match current {
            Value::Pair(pair) => pair.borrow().cdr.clone(),
            Value::EmptyList => {
                return Err(SchemeError::runtime(format!("'{name}' index out of range")))
            }
            other => {
                return Err(SchemeError::type_error(format!(
                    "'{name}' expected a proper list, got {other}"
                )))
            }
        };
    }

    match current {
        Value::Pair(pair) => Ok(pair),
        Value::EmptyList => Err(SchemeError::runtime(format!("'{name}' index out of range"))),
        other => Err(SchemeError::type_error(format!(
            "'{name}' expected a proper list, got {other}"
        ))),
    }
}
