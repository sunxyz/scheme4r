use super::*;

pub(super) fn install(env: &mut Environment) {
    define_builtin(env, "bytevector?", is_bytevector);
    define_builtin(env, "bytevector", bytevector);
    define_builtin(env, "make-bytevector", make_bytevector);
    define_builtin(env, "bytevector-length", bytevector_length);
    define_builtin(env, "bytevector-u8-ref", bytevector_u8_ref);
    define_builtin(env, "bytevector-u8-set!", bytevector_u8_set);
    define_builtin(env, "bytevector-copy", bytevector_copy);
    define_builtin(env, "bytevector-copy!", bytevector_copy_in_place);
    define_builtin(env, "bytevector-append", bytevector_append);
    define_builtin(env, "utf8->string", utf8_to_string);
    define_builtin(env, "string->utf8", string_to_utf8);
}

fn is_bytevector(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("bytevector?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::ByteVector(_))))
}

fn bytevector(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    let mut bytes = Vec::with_capacity(args.len());
    for value in args {
        bytes.push(expect_byte("bytevector", value)?);
    }
    Ok(Value::bytevector(bytes))
}

fn make_bytevector(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 2 {
        return Err(SchemeError::arity(
            "'make-bytevector' expects 1 or 2 arguments",
        ));
    }
    let len = expect_index("make-bytevector", &args[0])?;
    let fill = match args.get(1) {
        Some(value) => expect_byte("make-bytevector", value)?,
        None => 0,
    };
    Ok(Value::bytevector(vec![fill; len]))
}

fn bytevector_length(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("bytevector-length", args, 1)?;
    let bytes = expect_bytevector("bytevector-length", &args[0])?;
    let len = bytes.borrow().len() as i64;
    Ok(Value::Number(len))
}

fn bytevector_u8_ref(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("bytevector-u8-ref", args, 2)?;
    let bytes = expect_bytevector("bytevector-u8-ref", &args[0])?;
    let index = expect_index("bytevector-u8-ref", &args[1])?;
    let value = bytes
        .borrow()
        .get(index)
        .copied()
        .map(|value| Value::Number(i64::from(value)))
        .ok_or_else(|| SchemeError::runtime("'bytevector-u8-ref' index out of range"))?;
    Ok(value)
}

fn bytevector_u8_set(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("bytevector-u8-set!", args, 3)?;
    let bytes = expect_bytevector("bytevector-u8-set!", &args[0])?;
    let index = expect_index("bytevector-u8-set!", &args[1])?;
    let value = expect_byte("bytevector-u8-set!", &args[2])?;
    let mut bytes = bytes.borrow_mut();
    let slot = bytes
        .get_mut(index)
        .ok_or_else(|| SchemeError::runtime("'bytevector-u8-set!' index out of range"))?;
    *slot = value;
    Ok(Value::Unspecified)
}

fn bytevector_copy(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 3 {
        return Err(SchemeError::arity(
            "'bytevector-copy' expects 1 to 3 arguments",
        ));
    }

    let bytes = expect_bytevector("bytevector-copy", &args[0])?;
    let copied = {
        let bytes_ref = bytes.borrow();
        let (start, end) =
            parse_range("bytevector-copy", bytes_ref.len(), args.get(1), args.get(2))?;
        bytes_ref[start..end].to_vec()
    };
    Ok(Value::bytevector(copied))
}

fn bytevector_copy_in_place(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() < 3 || args.len() > 5 {
        return Err(SchemeError::arity(
            "'bytevector-copy!' expects 3 to 5 arguments",
        ));
    }

    let target = expect_bytevector("bytevector-copy!", &args[0])?;
    let at = expect_index("bytevector-copy!", &args[1])?;
    let source = expect_bytevector("bytevector-copy!", &args[2])?;
    let copied = {
        let source_ref = source.borrow();
        let (start, end) = parse_range(
            "bytevector-copy!",
            source_ref.len(),
            args.get(3),
            args.get(4),
        )?;
        source_ref[start..end].to_vec()
    };

    let mut target_ref = target.borrow_mut();
    if at > target_ref.len() || copied.len() > target_ref.len().saturating_sub(at) {
        return Err(SchemeError::runtime(
            "'bytevector-copy!' range out of bounds",
        ));
    }

    for (offset, value) in copied.into_iter().enumerate() {
        target_ref[at + offset] = value;
    }
    Ok(Value::Unspecified)
}

fn bytevector_append(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    let mut bytes = Vec::new();
    for value in args {
        let bytevector = expect_bytevector("bytevector-append", value)?;
        bytes.extend(bytevector.borrow().iter().copied());
    }
    Ok(Value::bytevector(bytes))
}

fn utf8_to_string(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 3 {
        return Err(SchemeError::arity(
            "'utf8->string' expects 1 to 3 arguments",
        ));
    }

    let bytes = expect_bytevector("utf8->string", &args[0])?;
    let text = {
        let bytes_ref = bytes.borrow();
        let (start, end) = parse_range("utf8->string", bytes_ref.len(), args.get(1), args.get(2))?;
        String::from_utf8(bytes_ref[start..end].to_vec())
            .map_err(|_| SchemeError::runtime("'utf8->string' expected valid UTF-8 bytes"))?
    };
    Ok(Value::string(text))
}

fn string_to_utf8(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 3 {
        return Err(SchemeError::arity(
            "'string->utf8' expects 1 to 3 arguments",
        ));
    }

    let text = expect_string("string->utf8", &args[0])?;
    let chars = text.chars().collect::<Vec<_>>();
    let (start, end) = parse_range("string->utf8", chars.len(), args.get(1), args.get(2))?;
    let slice = chars[start..end].iter().collect::<String>();
    Ok(Value::bytevector(slice.into_bytes()))
}
