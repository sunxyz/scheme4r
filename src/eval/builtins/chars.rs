use super::*;

pub(super) fn install(env: &mut Environment) {
    define_builtin(env, "char?", is_char);
    define_builtin(env, "char->integer", char_to_integer);
    define_builtin(env, "integer->char", integer_to_char);
    define_builtin(env, "char=?", char_equal);
    define_builtin(env, "char-ci=?", char_ci_equal);
    define_builtin(env, "char<?", char_less_than);
    define_builtin(env, "char-ci<?", char_ci_less_than);
    define_builtin(env, "char>?", char_greater_than);
    define_builtin(env, "char-ci>?", char_ci_greater_than);
    define_builtin(env, "char<=?", char_less_than_or_equal);
    define_builtin(env, "char-ci<=?", char_ci_less_than_or_equal);
    define_builtin(env, "char>=?", char_greater_than_or_equal);
    define_builtin(env, "char-ci>=?", char_ci_greater_than_or_equal);
    define_builtin(env, "char-alphabetic?", char_alphabetic);
    define_builtin(env, "char-numeric?", char_numeric);
    define_builtin(env, "char-whitespace?", char_whitespace);
    define_builtin(env, "char-upper-case?", char_upper_case);
    define_builtin(env, "char-lower-case?", char_lower_case);
    define_builtin(env, "char-upcase", char_upcase);
    define_builtin(env, "char-downcase", char_downcase);
    define_builtin(env, "char-foldcase", char_foldcase);
    define_builtin(env, "digit-value", digit_value);
}

fn is_char(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("char?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::Character(_))))
}

fn char_to_integer(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("char->integer", args, 1)?;
    Ok(Value::Number(i64::from(u32::from(expect_char(
        "char->integer",
        &args[0],
    )?))))
}

fn integer_to_char(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("integer->char", args, 1)?;
    let number = expect_number("integer->char", &args[0])?;
    let scalar = u32::try_from(number).map_err(|_| {
        SchemeError::type_error("'integer->char' expected a non-negative scalar value")
    })?;
    let ch = char::from_u32(scalar).ok_or_else(|| {
        SchemeError::type_error("'integer->char' expected a valid Unicode scalar value")
    })?;
    Ok(Value::Character(ch))
}

fn char_equal(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_char_chain(
        "char=?",
        args,
        |left, right| left == right,
    )?))
}

fn char_less_than(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_char_chain(
        "char<?",
        args,
        |left, right| left < right,
    )?))
}

fn char_ci_equal(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_char_chain_ci(
        "char-ci=?",
        args,
        |left, right| left == right,
    )?))
}

fn char_ci_less_than(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_char_chain_ci(
        "char-ci<?",
        args,
        |left, right| left < right,
    )?))
}

fn char_greater_than(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_char_chain(
        "char>?",
        args,
        |left, right| left > right,
    )?))
}

fn char_ci_greater_than(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_char_chain_ci(
        "char-ci>?",
        args,
        |left, right| left > right,
    )?))
}

fn char_less_than_or_equal(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_char_chain(
        "char<=?",
        args,
        |left, right| left <= right,
    )?))
}

fn char_ci_less_than_or_equal(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_char_chain_ci(
        "char-ci<=?",
        args,
        |left, right| left <= right,
    )?))
}

fn char_greater_than_or_equal(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_char_chain(
        "char>=?",
        args,
        |left, right| left >= right,
    )?))
}

fn char_ci_greater_than_or_equal(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::Boolean(compare_char_chain_ci(
        "char-ci>=?",
        args,
        |left, right| left >= right,
    )?))
}

fn char_alphabetic(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("char-alphabetic?", args, 1)?;
    Ok(Value::Boolean(
        expect_char("char-alphabetic?", &args[0])?.is_alphabetic(),
    ))
}

fn char_numeric(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("char-numeric?", args, 1)?;
    Ok(Value::Boolean(
        expect_char("char-numeric?", &args[0])?.is_numeric(),
    ))
}

fn char_whitespace(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("char-whitespace?", args, 1)?;
    Ok(Value::Boolean(
        expect_char("char-whitespace?", &args[0])?.is_whitespace(),
    ))
}

fn char_upper_case(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("char-upper-case?", args, 1)?;
    Ok(Value::Boolean(
        expect_char("char-upper-case?", &args[0])?.is_uppercase(),
    ))
}

fn char_lower_case(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("char-lower-case?", args, 1)?;
    Ok(Value::Boolean(
        expect_char("char-lower-case?", &args[0])?.is_lowercase(),
    ))
}

fn char_upcase(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("char-upcase", args, 1)?;
    Ok(Value::Character(map_case_char(
        expect_char("char-upcase", &args[0])?,
        true,
    )))
}

fn char_downcase(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("char-downcase", args, 1)?;
    Ok(Value::Character(map_case_char(
        expect_char("char-downcase", &args[0])?,
        false,
    )))
}

fn char_foldcase(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("char-foldcase", args, 1)?;
    Ok(Value::Character(fold_case_char(expect_char(
        "char-foldcase",
        &args[0],
    )?)))
}

fn digit_value(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("digit-value", args, 1)?;
    let ch = expect_char("digit-value", &args[0])?;
    Ok(match ch.to_digit(10) {
        Some(value) => Value::Number(i64::from(value)),
        None => Value::Boolean(false),
    })
}

fn compare_char_chain<F>(name: &str, args: &[Value], cmp: F) -> Result<bool, SchemeError>
where
    F: Fn(char, char) -> bool,
{
    if args.len() < 2 {
        return Err(SchemeError::arity(format!(
            "'{name}' expects at least 2 arguments"
        )));
    }

    let mut previous = expect_char(name, &args[0])?;
    for value in &args[1..] {
        let current = expect_char(name, value)?;
        if !cmp(previous, current) {
            return Ok(false);
        }
        previous = current;
    }
    Ok(true)
}

fn compare_char_chain_ci<F>(name: &str, args: &[Value], cmp: F) -> Result<bool, SchemeError>
where
    F: Fn(char, char) -> bool,
{
    if args.len() < 2 {
        return Err(SchemeError::arity(format!(
            "'{name}' expects at least 2 arguments"
        )));
    }

    let mut previous = fold_case_char(expect_char(name, &args[0])?);
    for value in &args[1..] {
        let current = fold_case_char(expect_char(name, value)?);
        if !cmp(previous, current) {
            return Ok(false);
        }
        previous = current;
    }
    Ok(true)
}

fn map_case_char(ch: char, uppercase: bool) -> char {
    if uppercase {
        let mut mapped = ch.to_uppercase();
        let Some(first) = mapped.next() else {
            return ch;
        };
        if mapped.next().is_some() {
            ch
        } else {
            first
        }
    } else {
        let mut mapped = ch.to_lowercase();
        let Some(first) = mapped.next() else {
            return ch;
        };
        if mapped.next().is_some() {
            ch
        } else {
            first
        }
    }
}

fn fold_case_char(ch: char) -> char {
    map_case_char(ch, false)
}
