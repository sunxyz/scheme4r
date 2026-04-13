use super::*;
use crate::runtime::SchemeString;

pub(super) fn install(env: &mut Environment) {
    define_builtin(env, "string?", is_string);
    define_builtin(env, "string", string);
    define_builtin(env, "make-string", make_string);
    define_builtin(env, "string-length", string_length);
    define_builtin(env, "string-ref", string_ref);
    define_builtin(env, "string-set!", string_set);
    define_builtin(env, "substring", substring);
    define_builtin(env, "string-copy", string_copy);
    define_builtin(env, "string-copy!", string_copy_in_place);
    define_builtin(env, "string-fill!", string_fill);
    define_builtin(env, "string-append", string_append);
    define_builtin(env, "string=?", string_equal);
    define_builtin(env, "string-ci=?", string_ci_equal);
    define_builtin(env, "string<?", string_less_than);
    define_builtin(env, "string-ci<?", string_ci_less_than);
    define_builtin(env, "string>?", string_greater_than);
    define_builtin(env, "string-ci>?", string_ci_greater_than);
    define_builtin(env, "string<=?", string_less_than_or_equal);
    define_builtin(env, "string-ci<=?", string_ci_less_than_or_equal);
    define_builtin(env, "string>=?", string_greater_than_or_equal);
    define_builtin(env, "string-ci>=?", string_ci_greater_than_or_equal);
    define_builtin(env, "string-map", string_map);
    define_builtin(env, "string-for-each", string_for_each);
    define_builtin(env, "string->list", string_to_list);
    define_builtin(env, "list->string", list_to_string);
    define_builtin(env, "string->vector", string_to_vector);
    define_builtin(env, "vector->string", vector_to_string);
    define_builtin(env, "string-upcase", string_upcase);
    define_builtin(env, "string-downcase", string_downcase);
    define_builtin(env, "string-foldcase", string_foldcase);
}

fn is_string(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("string?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::String(_))))
}

fn string(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    let mut text = String::new();
    for value in args {
        text.push(expect_char("string", value)?);
    }
    Ok(Value::string(text))
}

fn make_string(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 2 {
        return Err(SchemeError::arity("'make-string' expects 1 or 2 arguments"));
    }

    let len = expect_index("make-string", &args[0])?;
    let fill = match args.get(1) {
        Some(value) => expect_char("make-string", value)?,
        None => '\0',
    };

    Ok(Value::string(
        std::iter::repeat(fill).take(len).collect::<String>(),
    ))
}

fn string_length(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("string-length", args, 1)?;
    let text = expect_string("string-length", &args[0])?;
    Ok(Value::Number(text.chars().count() as i64))
}

fn string_ref(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("string-ref", args, 2)?;
    let text = expect_string("string-ref", &args[0])?;
    let index = expect_index("string-ref", &args[1])?;
    let ch = char_at("string-ref", &text, index)?;
    Ok(Value::Character(ch))
}

fn string_set(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("string-set!", args, 3)?;
    let string = expect_string_ref("string-set!", &args[0])?;
    let index = expect_index("string-set!", &args[1])?;
    let replacement = expect_char("string-set!", &args[2])?;

    let mut chars = string_chars("string-set!", &string)?;
    let slot = chars
        .get_mut(index)
        .ok_or_else(|| SchemeError::runtime("'string-set!' index out of range"))?;
    *slot = replacement;
    *string.borrow_mut() = chars.into_iter().collect();
    Ok(Value::Unspecified)
}

fn substring(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("substring", args, 3)?;
    let text = expect_string("substring", &args[0])?;
    let start = expect_index("substring", &args[1])?;
    let end = expect_index("substring", &args[2])?;
    let chars = text.chars().collect::<Vec<_>>();

    if start > end || end > chars.len() {
        return Err(SchemeError::runtime("'substring' index out of range"));
    }

    Ok(Value::string(chars[start..end].iter().collect::<String>()))
}

fn string_copy(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 3 {
        return Err(SchemeError::arity("'string-copy' expects 1 to 3 arguments"));
    }
    let text = expect_string("string-copy", &args[0])?;
    let chars = text.chars().collect::<Vec<_>>();
    let (start, end) = parse_string_range("string-copy", chars.len(), args.get(1), args.get(2))?;
    Ok(Value::string(chars[start..end].iter().collect::<String>()))
}

fn string_copy_in_place(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() < 3 || args.len() > 5 {
        return Err(SchemeError::arity(
            "'string-copy!' expects 3 to 5 arguments",
        ));
    }

    let target = expect_string_ref("string-copy!", &args[0])?;
    let at = expect_index("string-copy!", &args[1])?;
    let source = expect_string("string-copy!", &args[2])?;
    let source_chars = source.chars().collect::<Vec<_>>();
    let (start, end) =
        parse_string_range("string-copy!", source_chars.len(), args.get(3), args.get(4))?;
    let copied = source_chars[start..end].to_vec();

    let mut target_chars = string_chars("string-copy!", &target)?;
    if at > target_chars.len() || copied.len() > target_chars.len().saturating_sub(at) {
        return Err(SchemeError::runtime("'string-copy!' index out of range"));
    }

    for (offset, ch) in copied.into_iter().enumerate() {
        target_chars[at + offset] = ch;
    }
    *target.borrow_mut() = target_chars.into_iter().collect();
    Ok(Value::Unspecified)
}

fn string_fill(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() < 2 || args.len() > 4 {
        return Err(SchemeError::arity(
            "'string-fill!' expects 2 to 4 arguments",
        ));
    }

    let string = expect_string_ref("string-fill!", &args[0])?;
    let fill = expect_char("string-fill!", &args[1])?;
    let mut chars = string_chars("string-fill!", &string)?;
    let (start, end) = parse_string_range("string-fill!", chars.len(), args.get(2), args.get(3))?;
    for slot in &mut chars[start..end] {
        *slot = fill;
    }
    *string.borrow_mut() = chars.into_iter().collect();
    Ok(Value::Unspecified)
}

fn string_append(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    let mut result = String::new();
    for value in args {
        result.push_str(&expect_string("string-append", value)?);
    }
    Ok(Value::string(result))
}

fn string_equal(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_string_chain(
        "string=?",
        args,
        |left, right| left == right,
    )?))
}

fn string_ci_equal(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_string_chain_ci(
        "string-ci=?",
        args,
        |left, right| left == right,
    )?))
}

fn string_less_than(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_string_chain(
        "string<?",
        args,
        |left, right| left < right,
    )?))
}

fn string_ci_less_than(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_string_chain_ci(
        "string-ci<?",
        args,
        |left, right| left < right,
    )?))
}

fn string_greater_than(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_string_chain(
        "string>?",
        args,
        |left, right| left > right,
    )?))
}

fn string_ci_greater_than(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_string_chain_ci(
        "string-ci>?",
        args,
        |left, right| left > right,
    )?))
}

fn string_less_than_or_equal(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_string_chain(
        "string<=?",
        args,
        |left, right| left <= right,
    )?))
}

fn string_ci_less_than_or_equal(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_string_chain_ci(
        "string-ci<=?",
        args,
        |left, right| left <= right,
    )?))
}

fn string_greater_than_or_equal(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_string_chain(
        "string>=?",
        args,
        |left, right| left >= right,
    )?))
}

fn string_ci_greater_than_or_equal(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_string_chain_ci(
        "string-ci>=?",
        args,
        |left, right| left >= right,
    )?))
}

fn string_map(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() < 2 {
        return Err(SchemeError::arity(
            "'string-map' expects a procedure and at least one string",
        ));
    }

    let procedure = args[0].clone();
    let char_vectors = collect_parallel_strings("string-map", &args[1..])?;
    let mut result = String::new();

    for index in 0..char_vectors[0].len() {
        let call_args = char_vectors
            .iter()
            .map(|chars| Value::Character(chars[index]))
            .collect::<Vec<_>>();
        let mapped = engine.apply(procedure.clone(), engine.current_env(), call_args)?;
        result.push(expect_char("string-map", &mapped)?);
    }

    Ok(Value::string(result))
}

fn string_for_each(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() < 2 {
        return Err(SchemeError::arity(
            "'string-for-each' expects a procedure and at least one string",
        ));
    }

    let procedure = args[0].clone();
    let char_vectors = collect_parallel_strings("string-for-each", &args[1..])?;

    for index in 0..char_vectors[0].len() {
        let call_args = char_vectors
            .iter()
            .map(|chars| Value::Character(chars[index]))
            .collect::<Vec<_>>();
        engine.apply(procedure.clone(), engine.current_env(), call_args)?;
    }

    Ok(Value::Unspecified)
}

fn string_to_list(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 3 {
        return Err(SchemeError::arity(
            "'string->list' expects 1 to 3 arguments",
        ));
    }
    let text = expect_string("string->list", &args[0])?;
    let chars = text.chars().collect::<Vec<_>>();
    let (start, end) = parse_string_range("string->list", chars.len(), args.get(1), args.get(2))?;
    Ok(Value::list(
        chars[start..end]
            .iter()
            .copied()
            .map(Value::Character)
            .collect(),
    ))
}

fn list_to_string(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("list->string", args, 1)?;
    let chars = expect_list("list->string", &args[0])?;
    let mut text = String::new();
    for value in chars {
        text.push(expect_char("list->string", &value)?);
    }
    Ok(Value::string(text))
}

fn string_to_vector(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 3 {
        return Err(SchemeError::arity(
            "'string->vector' expects 1 to 3 arguments",
        ));
    }
    let text = expect_string("string->vector", &args[0])?;
    let chars = text.chars().collect::<Vec<_>>();
    let (start, end) = parse_string_range("string->vector", chars.len(), args.get(1), args.get(2))?;
    Ok(Value::vector(
        chars[start..end]
            .iter()
            .copied()
            .map(Value::Character)
            .collect(),
    ))
}

fn vector_to_string(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 3 {
        return Err(SchemeError::arity(
            "'vector->string' expects 1 to 3 arguments",
        ));
    }
    let vector = expect_vector("vector->string", &args[0])?;
    let values = vector.borrow().clone();
    let (start, end) =
        parse_string_range("vector->string", values.len(), args.get(1), args.get(2))?;
    let mut text = String::new();
    for value in &values[start..end] {
        text.push(expect_char("vector->string", value)?);
    }
    Ok(Value::string(text))
}

fn string_upcase(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("string-upcase", args, 1)?;
    Ok(Value::string(
        expect_string("string-upcase", &args[0])?.to_uppercase(),
    ))
}

fn string_downcase(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("string-downcase", args, 1)?;
    Ok(Value::string(
        expect_string("string-downcase", &args[0])?.to_lowercase(),
    ))
}

fn string_foldcase(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("string-foldcase", args, 1)?;
    Ok(Value::string(
        expect_string("string-foldcase", &args[0])?.to_lowercase(),
    ))
}

fn char_at(name: &str, text: &str, index: usize) -> Result<char, SchemeError> {
    text.chars()
        .nth(index)
        .ok_or_else(|| SchemeError::runtime(format!("'{name}' index out of range")))
}

fn compare_string_chain<F>(name: &str, args: &[Value], cmp: F) -> Result<bool, SchemeError>
where
    F: Fn(&str, &str) -> bool,
{
    if args.len() < 2 {
        return Err(SchemeError::arity(format!(
            "'{name}' expects at least 2 arguments"
        )));
    }

    let mut previous = expect_string(name, &args[0])?;
    for value in &args[1..] {
        let current = expect_string(name, value)?;
        if !cmp(&previous, &current) {
            return Ok(false);
        }
        previous.clear();
        previous.push_str(&current);
    }
    Ok(true)
}

fn compare_string_chain_ci<F>(name: &str, args: &[Value], cmp: F) -> Result<bool, SchemeError>
where
    F: Fn(&str, &str) -> bool,
{
    if args.len() < 2 {
        return Err(SchemeError::arity(format!(
            "'{name}' expects at least 2 arguments"
        )));
    }

    let mut previous = expect_string(name, &args[0])?.to_lowercase();
    for value in &args[1..] {
        let current = expect_string(name, value)?.to_lowercase();
        if !cmp(&previous, &current) {
            return Ok(false);
        }
        previous = current;
    }
    Ok(true)
}

fn parse_string_range(
    name: &str,
    len: usize,
    start_arg: Option<&Value>,
    end_arg: Option<&Value>,
) -> Result<(usize, usize), SchemeError> {
    let start = match start_arg {
        Some(value) => expect_index(name, value)?,
        None => 0,
    };
    let end = match end_arg {
        Some(value) => expect_index(name, value)?,
        None => len,
    };

    if start > end || end > len {
        return Err(SchemeError::runtime(format!("'{name}' index out of range")));
    }

    Ok((start, end))
}

fn collect_parallel_strings(name: &str, args: &[Value]) -> Result<Vec<Vec<char>>, SchemeError> {
    let mut char_vectors = Vec::with_capacity(args.len());
    let mut expected_len = None;

    for value in args {
        let chars = expect_string(name, value)?.chars().collect::<Vec<_>>();
        match expected_len {
            Some(len) if len != chars.len() => {
                return Err(SchemeError::runtime(format!(
                    "'{name}' expected strings of equal length"
                )));
            }
            None => expected_len = Some(chars.len()),
            _ => {}
        }
        char_vectors.push(chars);
    }

    Ok(char_vectors)
}

fn string_chars(name: &str, string: &SchemeString) -> Result<Vec<char>, SchemeError> {
    let text = string.to_plain_string();
    if text.chars().count() > i64::MAX as usize {
        return Err(SchemeError::runtime(format!(
            "'{name}' string length overflow"
        )));
    }
    Ok(text.chars().collect())
}
