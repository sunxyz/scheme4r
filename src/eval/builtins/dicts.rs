use std::collections::BTreeMap;

use super::*;
use crate::runtime::DictKey;

pub(super) fn install(env: &mut Environment) {
    define_builtin(env, "dict?", is_dict);
    define_builtin(env, "make-dict", make_dict);
    define_builtin(env, "dict", dict_from_pairs);
    define_builtin(env, "dict-size", dict_size);
    define_builtin(env, "dict-has-key?", dict_has_key);
    define_builtin(env, "dict-ref", dict_ref);
    define_builtin(env, "dict-set!", dict_set);
    define_builtin(env, "dict-delete!", dict_delete);
    define_builtin(env, "dict-clear!", dict_clear);
    define_builtin(env, "dict-keys", dict_keys);
}

fn is_dict(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("dict?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::Dict(_))))
}

fn make_dict(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("make-dict", args, 0)?;
    Ok(Value::dict(BTreeMap::new()))
}

fn dict_from_pairs(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() % 2 != 0 {
        return Err(SchemeError::arity(
            "'dict' expects an even number of key/value arguments",
        ));
    }

    let mut map = BTreeMap::new();
    let mut index = 0;
    while index < args.len() {
        let key = expect_dict_key("dict", &args[index])?;
        let value = args[index + 1].clone();
        map.insert(key, value);
        index += 2;
    }

    Ok(Value::dict(map))
}

fn dict_size(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("dict-size", args, 1)?;
    let dict = expect_dict("dict-size", &args[0])?;
    let len = dict.borrow().len() as i64;
    Ok(Value::Number(len))
}

fn dict_has_key(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("dict-has-key?", args, 2)?;
    let dict = expect_dict("dict-has-key?", &args[0])?;
    let key = expect_dict_key("dict-has-key?", &args[1])?;
    let has_key = dict.borrow().contains_key(&key);
    Ok(Value::Boolean(has_key))
}

fn dict_ref(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() != 2 && args.len() != 3 {
        return Err(SchemeError::arity("'dict-ref' expects 2 or 3 arguments"));
    }

    let dict = expect_dict("dict-ref", &args[0])?;
    let key = expect_dict_key("dict-ref", &args[1])?;
    let maybe_value = dict.borrow().get(&key).cloned();
    match maybe_value {
        Some(value) => Ok(value),
        None => match args.get(2) {
            Some(default) => Ok(default.clone()),
            None => Err(SchemeError::runtime("'dict-ref' key does not exist")),
        },
    }
}

fn dict_set(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("dict-set!", args, 3)?;
    let dict = expect_dict("dict-set!", &args[0])?;
    let key = expect_dict_key("dict-set!", &args[1])?;
    dict.borrow_mut().insert(key, args[2].clone());
    Ok(Value::Unspecified)
}

fn dict_delete(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("dict-delete!", args, 2)?;
    let dict = expect_dict("dict-delete!", &args[0])?;
    let key = expect_dict_key("dict-delete!", &args[1])?;
    let removed = dict.borrow_mut().remove(&key).is_some();
    Ok(Value::Boolean(removed))
}

fn dict_clear(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("dict-clear!", args, 1)?;
    let dict = expect_dict("dict-clear!", &args[0])?;
    dict.borrow_mut().clear();
    Ok(Value::Unspecified)
}

fn dict_keys(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("dict-keys", args, 1)?;
    let dict = expect_dict("dict-keys", &args[0])?;
    let keys = dict
        .borrow()
        .keys()
        .map(DictKey::to_value)
        .collect::<Vec<_>>();
    Ok(Value::list(keys))
}

fn expect_dict_key(name: &str, value: &Value) -> Result<DictKey, SchemeError> {
    DictKey::try_from_value(value).ok_or_else(|| {
        SchemeError::type_error(format!(
            "'{name}' key must be boolean/number/character/string/symbol/()"
        ))
    })
}
