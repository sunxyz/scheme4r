use super::*;

pub(super) fn install(env: &mut Environment) {
    define_builtin(env, "vector?", is_vector);
    define_builtin(env, "vector", vector);
    define_builtin(env, "make-vector", make_vector);
    define_builtin(env, "vector-length", vector_length);
    define_builtin(env, "vector-ref", vector_ref);
    define_builtin(env, "vector-set!", vector_set);
    define_builtin(env, "vector-fill!", vector_fill);
    define_builtin(env, "vector-copy", vector_copy);
    define_builtin(env, "vector-copy!", vector_copy_in_place);
    define_builtin(env, "vector-append", vector_append);
    define_builtin(env, "vector-map", vector_map);
    define_builtin(env, "vector-for-each", vector_for_each);
    define_builtin(env, "vector->list", vector_to_list);
    define_builtin(env, "list->vector", list_to_vector);
}

fn is_vector(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("vector?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::Vector(_))))
}

fn vector(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    Ok(Value::vector(args.to_vec()))
}

fn make_vector(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 2 {
        return Err(SchemeError::arity("'make-vector' expects 1 or 2 arguments"));
    }
    let len = expect_index("make-vector", &args[0])?;
    let fill = args.get(1).cloned().unwrap_or(Value::Unspecified);
    Ok(Value::vector(vec![fill; len]))
}

fn vector_length(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("vector-length", args, 1)?;
    let vector = expect_vector("vector-length", &args[0])?;
    let len = vector.borrow().len() as i64;
    Ok(Value::Number(len))
}

fn vector_ref(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("vector-ref", args, 2)?;
    let vector = expect_vector("vector-ref", &args[0])?;
    let index = expect_index("vector-ref", &args[1])?;
    let value = vector
        .borrow()
        .get(index)
        .cloned()
        .ok_or_else(|| SchemeError::runtime("'vector-ref' index out of range"))?;
    Ok(value)
}

fn vector_set(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("vector-set!", args, 3)?;
    let vector = expect_vector("vector-set!", &args[0])?;
    let index = expect_index("vector-set!", &args[1])?;
    let mut vector = vector.borrow_mut();
    let slot = vector
        .get_mut(index)
        .ok_or_else(|| SchemeError::runtime("'vector-set!' index out of range"))?;
    *slot = args[2].clone();
    Ok(Value::Unspecified)
}

fn vector_fill(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() < 2 || args.len() > 4 {
        return Err(SchemeError::arity(
            "'vector-fill!' expects 2 to 4 arguments",
        ));
    }
    let vector = expect_vector("vector-fill!", &args[0])?;
    let fill = args[1].clone();
    let (start, end) = {
        let vector_ref = vector.borrow();
        parse_range("vector-fill!", vector_ref.len(), args.get(2), args.get(3))?
    };

    let mut vector_ref = vector.borrow_mut();
    for slot in &mut vector_ref[start..end] {
        *slot = fill.clone();
    }
    Ok(Value::Unspecified)
}

fn vector_copy(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 3 {
        return Err(SchemeError::arity("'vector-copy' expects 1 to 3 arguments"));
    }
    let vector = expect_vector("vector-copy", &args[0])?;
    let copied = {
        let vector_ref = vector.borrow();
        let (start, end) = parse_range("vector-copy", vector_ref.len(), args.get(1), args.get(2))?;
        vector_ref[start..end].to_vec()
    };
    Ok(Value::vector(copied))
}

fn vector_copy_in_place(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() < 3 || args.len() > 5 {
        return Err(SchemeError::arity(
            "'vector-copy!' expects 3 to 5 arguments",
        ));
    }

    let target = expect_vector("vector-copy!", &args[0])?;
    let at = expect_index("vector-copy!", &args[1])?;
    let source = expect_vector("vector-copy!", &args[2])?;
    let copied = {
        let source_ref = source.borrow();
        let (start, end) = parse_range("vector-copy!", source_ref.len(), args.get(3), args.get(4))?;
        source_ref[start..end].to_vec()
    };

    let mut target_ref = target.borrow_mut();
    if at > target_ref.len() || copied.len() > target_ref.len().saturating_sub(at) {
        return Err(SchemeError::runtime("'vector-copy!' range out of bounds"));
    }

    for (offset, value) in copied.into_iter().enumerate() {
        target_ref[at + offset] = value;
    }
    Ok(Value::Unspecified)
}

fn vector_append(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    let mut values = Vec::new();
    for value in args {
        let vector = expect_vector("vector-append", value)?;
        values.extend(vector.borrow().iter().cloned());
    }
    Ok(Value::vector(values))
}

fn vector_map(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() < 2 {
        return Err(SchemeError::arity(
            "'vector-map' expects a procedure and at least one vector",
        ));
    }

    let procedure = args[0].clone();
    let value_vectors = collect_parallel_vectors("vector-map", &args[1..])?;
    let mut results = Vec::with_capacity(value_vectors[0].len());

    for index in 0..value_vectors[0].len() {
        let call_args = value_vectors
            .iter()
            .map(|values| values[index].clone())
            .collect::<Vec<_>>();
        results.push(engine.apply(procedure.clone(), engine.current_env(), call_args)?);
    }

    Ok(Value::vector(results))
}

fn vector_for_each(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() < 2 {
        return Err(SchemeError::arity(
            "'vector-for-each' expects a procedure and at least one vector",
        ));
    }

    let procedure = args[0].clone();
    let value_vectors = collect_parallel_vectors("vector-for-each", &args[1..])?;

    for index in 0..value_vectors[0].len() {
        let call_args = value_vectors
            .iter()
            .map(|values| values[index].clone())
            .collect::<Vec<_>>();
        engine.apply(procedure.clone(), engine.current_env(), call_args)?;
    }

    Ok(Value::Unspecified)
}

fn vector_to_list(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 3 {
        return Err(SchemeError::arity(
            "'vector->list' expects 1 to 3 arguments",
        ));
    }
    let vector = expect_vector("vector->list", &args[0])?;
    let values = vector.borrow().clone();
    let (start, end) = parse_range("vector->list", values.len(), args.get(1), args.get(2))?;
    Ok(Value::list(values[start..end].to_vec()))
}

fn list_to_vector(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("list->vector", args, 1)?;
    Ok(Value::vector(expect_list("list->vector", &args[0])?))
}

fn collect_parallel_vectors(name: &str, args: &[Value]) -> Result<Vec<Vec<Value>>, SchemeError> {
    let mut value_vectors = Vec::with_capacity(args.len());
    let mut expected_len = None;

    for value in args {
        let vector = expect_vector(name, value)?;
        let values = vector.borrow().clone();
        match expected_len {
            Some(len) if len != values.len() => {
                return Err(SchemeError::runtime(format!(
                    "'{name}' expected vectors of equal length"
                )));
            }
            None => expected_len = Some(values.len()),
            _ => {}
        }
        value_vectors.push(values);
    }

    Ok(value_vectors)
}
