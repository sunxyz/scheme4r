use std::fs;

use super::*;
use crate::runtime::Port;

pub(super) fn install(env: &mut Environment) {
    define_builtin(env, "read", read);
    define_builtin(env, "read-char", read_char);
    define_builtin(env, "read-line", read_line);
    define_builtin(env, "peek-char", peek_char);
    define_builtin(env, "char-ready?", char_ready);
    define_builtin(env, "read-string", read_string);
    define_builtin(env, "eval", eval_builtin);
    define_builtin(env, "load", load);
    define_builtin(env, "eof-object?", eof_object_predicate);
    define_builtin(env, "port?", port_predicate);
    define_builtin(env, "input-port?", input_port_predicate);
    define_builtin(env, "output-port?", output_port_predicate);
    define_builtin(env, "textual-port?", textual_port_predicate);
    define_builtin(env, "binary-port?", binary_port_predicate);
    define_builtin(env, "input-port-open?", input_port_open);
    define_builtin(env, "output-port-open?", output_port_open);
    define_builtin(env, "current-input-port", current_input_port);
    define_builtin(env, "current-output-port", current_output_port);
    define_builtin(env, "current-error-port", current_error_port);
    define_builtin(env, "open-input-string", open_input_string);
    define_builtin(env, "open-output-string", open_output_string);
    define_builtin(env, "get-output-string", get_output_string);
    define_builtin(env, "open-input-bytevector", open_input_bytevector);
    define_builtin(env, "open-output-bytevector", open_output_bytevector);
    define_builtin(env, "get-output-bytevector", get_output_bytevector);
    define_builtin(env, "open-input-file", open_input_file);
    define_builtin(env, "open-output-file", open_output_file);
    define_builtin(env, "close-port", close_port);
    define_builtin(env, "close-input-port", close_input_port);
    define_builtin(env, "close-output-port", close_output_port);
    define_builtin(env, "call-with-port", call_with_port);
    define_builtin(env, "read-u8", read_u8);
    define_builtin(env, "peek-u8", peek_u8);
    define_builtin(env, "u8-ready?", u8_ready);
    define_builtin(env, "read-bytevector", read_bytevector);
    define_builtin(env, "read-bytevector!", read_bytevector_in_place);
    define_builtin(env, "display", display);
    define_builtin(env, "write", write);
    define_builtin(env, "write-shared", write);
    define_builtin(env, "write-simple", write);
    define_builtin(env, "write-char", write_char);
    define_builtin(env, "write-string", write_string);
    define_builtin(env, "write-u8", write_u8);
    define_builtin(env, "write-bytevector", write_bytevector);
    define_builtin(env, "flush-output-port", flush_output_port);
    define_builtin(env, "newline", newline);
}

fn read(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() > 1 {
        return Err(SchemeError::arity("'read' expects 0 or 1 arguments"));
    }

    let port = match args.first() {
        Some(value) => expect_input_port("read", value)?,
        None => engine.current_input_port(),
    };

    let datum = port.borrow_mut().read_datum()?;
    Ok(match datum {
        Some(datum) => Value::from_datum(&datum),
        None => Value::EofObject,
    })
}

fn read_char(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() > 1 {
        return Err(SchemeError::arity("'read-char' expects 0 or 1 arguments"));
    }

    let port = match args.first() {
        Some(value) => expect_input_port("read-char", value)?,
        None => engine.current_input_port(),
    };

    let read = port.borrow_mut().read_char()?;
    Ok(match read {
        Some(ch) => Value::Character(ch),
        None => Value::EofObject,
    })
}

fn peek_char(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() > 1 {
        return Err(SchemeError::arity("'peek-char' expects 0 or 1 arguments"));
    }

    let port = match args.first() {
        Some(value) => expect_input_port("peek-char", value)?,
        None => engine.current_input_port(),
    };

    let peeked = port.borrow_mut().peek_char()?;
    Ok(match peeked {
        Some(ch) => Value::Character(ch),
        None => Value::EofObject,
    })
}

fn char_ready(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() > 1 {
        return Err(SchemeError::arity("'char-ready?' expects 0 or 1 arguments"));
    }
    let port = optional_input_port(engine, args.first(), "char-ready?")?;
    let ready = port.borrow().char_ready()?;
    Ok(Value::Boolean(ready))
}

fn read_line(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() > 1 {
        return Err(SchemeError::arity("'read-line' expects 0 or 1 arguments"));
    }

    let port = match args.first() {
        Some(value) => expect_input_port("read-line", value)?,
        None => engine.current_input_port(),
    };

    let mut buffer = String::new();
    loop {
        match port.borrow_mut().read_char()? {
            Some('\n') => return Ok(Value::string(buffer)),
            Some(ch) => buffer.push(ch),
            None if buffer.is_empty() => return Ok(Value::EofObject),
            None => return Ok(Value::string(buffer)),
        }
    }
}

fn read_string(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 2 {
        return Err(SchemeError::arity("'read-string' expects 1 or 2 arguments"));
    }
    let count = expect_index("read-string", &args[0])?;
    let port = optional_input_port(engine, args.get(1), "read-string")?;
    let read = port.borrow_mut().read_string(count)?;
    match read {
        Some(text) => Ok(Value::string(text)),
        None => Ok(Value::EofObject),
    }
}

fn eof_object_predicate(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("eof-object?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::EofObject)))
}

fn eval_builtin(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("eval", args, 1)?;
    let env = engine.current_env();
    match &args[0] {
        Value::String(source) => engine.run_in_env(&source.to_plain_string(), env),
        value => {
            let datum = value.to_datum()?;
            engine.eval_datum(&datum, env)
        }
    }
}

fn load(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("load", args, 1)?;
    let path = expect_string("load", &args[0])?;
    let source = fs::read_to_string(&path)
        .map_err(|err| SchemeError::io(format!("failed to load '{path}': {err}")))?;
    engine.run_in_env(&source, engine.current_env())
}

fn current_input_port(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("current-input-port", args, 0)?;
    Ok(Value::port(engine.current_input_port()))
}

fn current_output_port(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("current-output-port", args, 0)?;
    Ok(Value::port(engine.current_output_port()))
}

fn current_error_port(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("current-error-port", args, 0)?;
    Ok(Value::port(engine.current_error_port()))
}

fn open_input_string(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("open-input-string", args, 1)?;
    let source = expect_string("open-input-string", &args[0])?;
    Ok(Value::port(Port::open_input_string(&source)?))
}

fn open_output_string(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("open-output-string", args, 0)?;
    Ok(Value::port(Port::open_output_string()))
}

fn get_output_string(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("get-output-string", args, 1)?;
    let port = expect_output_port("get-output-string", &args[0])?;
    let text = port.borrow().output_string()?;
    Ok(Value::string(text))
}

fn open_input_bytevector(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("open-input-bytevector", args, 1)?;
    let bytes = expect_bytevector("open-input-bytevector", &args[0])?;
    let snapshot = bytes.borrow().clone();
    Ok(Value::port(Port::open_input_bytevector(&snapshot)))
}

fn open_output_bytevector(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("open-output-bytevector", args, 0)?;
    Ok(Value::port(Port::open_output_bytevector()))
}

fn get_output_bytevector(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("get-output-bytevector", args, 1)?;
    let port = expect_output_port("get-output-bytevector", &args[0])?;
    let bytes = port.borrow().output_bytevector()?;
    Ok(Value::bytevector(bytes))
}

fn open_input_file(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("open-input-file", args, 1)?;
    let path = expect_string("open-input-file", &args[0])?;
    Ok(Value::port(Port::open_input_file(&path)?))
}

fn open_output_file(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("open-output-file", args, 1)?;
    let path = expect_string("open-output-file", &args[0])?;
    Ok(Value::port(Port::open_output_file(&path)?))
}

fn close_port(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("close-port", args, 1)?;
    let port = expect_port("close-port", &args[0])?;
    port.borrow_mut().close();
    Ok(Value::Unspecified)
}

fn close_input_port(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("close-input-port", args, 1)?;
    let port = expect_input_port("close-input-port", &args[0])?;
    port.borrow_mut().close();
    Ok(Value::Unspecified)
}

fn close_output_port(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("close-output-port", args, 1)?;
    let port = expect_output_port("close-output-port", &args[0])?;
    port.borrow_mut().close();
    Ok(Value::Unspecified)
}

fn call_with_port(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("call-with-port", args, 2)?;
    let port = expect_port("call-with-port", &args[0])?;
    let result = engine.apply(args[1].clone(), engine.current_env(), vec![args[0].clone()]);
    port.borrow_mut().close();
    result
}

fn read_u8(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() > 1 {
        return Err(SchemeError::arity("'read-u8' expects 0 or 1 arguments"));
    }
    let port = optional_input_port(engine, args.first(), "read-u8")?;
    let read = port.borrow_mut().read_u8()?;
    Ok(match read {
        Some(byte) => Value::Number(i64::from(byte)),
        None => Value::EofObject,
    })
}

fn peek_u8(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() > 1 {
        return Err(SchemeError::arity("'peek-u8' expects 0 or 1 arguments"));
    }
    let port = optional_input_port(engine, args.first(), "peek-u8")?;
    let peeked = port.borrow_mut().peek_u8()?;
    Ok(match peeked {
        Some(byte) => Value::Number(i64::from(byte)),
        None => Value::EofObject,
    })
}

fn u8_ready(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() > 1 {
        return Err(SchemeError::arity("'u8-ready?' expects 0 or 1 arguments"));
    }
    let port = optional_input_port(engine, args.first(), "u8-ready?")?;
    let ready = port.borrow().u8_ready()?;
    Ok(Value::Boolean(ready))
}

fn read_bytevector(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 2 {
        return Err(SchemeError::arity(
            "'read-bytevector' expects 1 or 2 arguments",
        ));
    }
    let count = expect_index("read-bytevector", &args[0])?;
    let port = optional_input_port(engine, args.get(1), "read-bytevector")?;
    let read = port.borrow_mut().read_bytes(count)?;
    match read {
        Some(bytes) => Ok(Value::bytevector(bytes)),
        None => Ok(Value::EofObject),
    }
}

fn read_bytevector_in_place(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 4 {
        return Err(SchemeError::arity(
            "'read-bytevector!' expects 1 to 4 arguments",
        ));
    }

    let target = expect_bytevector("read-bytevector!", &args[0])?;
    let port = optional_input_port(engine, args.get(1), "read-bytevector!")?;
    let (start, end) = {
        let target_ref = target.borrow();
        parse_range(
            "read-bytevector!",
            target_ref.len(),
            args.get(2),
            args.get(3),
        )?
    };

    if start == end {
        return Ok(Value::Number(0));
    }

    let mut buffer = vec![0_u8; end - start];
    let read = port.borrow_mut().read_bytes_into(&mut buffer)?;
    match read {
        Some(count) => {
            target.borrow_mut()[start..start + count].copy_from_slice(&buffer[..count]);
            Ok(Value::Number(count as i64))
        }
        None => Ok(Value::EofObject),
    }
}

fn display(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 2 {
        return Err(SchemeError::arity("'display' expects 1 or 2 arguments"));
    }
    let port = optional_output_port(engine, args.get(1), "display")?;
    let text = match &args[0] {
        Value::String(text) => text.to_plain_string(),
        value => value.to_string(),
    };
    port.borrow_mut().write_str(&text)?;
    Ok(Value::Unspecified)
}

fn write(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 2 {
        return Err(SchemeError::arity("'write' expects 1 or 2 arguments"));
    }
    let port = optional_output_port(engine, args.get(1), "write")?;
    port.borrow_mut().write_str(&args[0].to_string())?;
    Ok(Value::Unspecified)
}

fn write_char(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 2 {
        return Err(SchemeError::arity("'write-char' expects 1 or 2 arguments"));
    }
    let port = optional_output_port(engine, args.get(1), "write-char")?;
    let ch = expect_char("write-char", &args[0])?;
    let mut encoded = [0_u8; 4];
    port.borrow_mut().write_str(ch.encode_utf8(&mut encoded))?;
    Ok(Value::Unspecified)
}

fn write_string(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 4 {
        return Err(SchemeError::arity(
            "'write-string' expects 1 to 4 arguments",
        ));
    }
    let text = expect_string("write-string", &args[0])?;
    let chars = text.chars().collect::<Vec<_>>();
    let port_index = if args.len() >= 2 && matches!(args[1], Value::Port(_)) {
        Some(1)
    } else {
        None
    };
    let port = optional_output_port(engine, port_index.and_then(|i| args.get(i)), "write-string")?;
    let start_arg = match port_index {
        Some(_) => args.get(2),
        None => args.get(1),
    };
    let end_arg = match port_index {
        Some(_) => args.get(3),
        None => args.get(2),
    };
    let start = start_arg
        .map(|value| expect_index("write-string", value))
        .transpose()?
        .unwrap_or(0);
    let end = end_arg
        .map(|value| expect_index("write-string", value))
        .transpose()?
        .unwrap_or(chars.len());
    if start > end || end > chars.len() {
        return Err(SchemeError::runtime("'write-string' range out of bounds"));
    }
    let slice = chars[start..end].iter().collect::<String>();
    port.borrow_mut().write_str(&slice)?;
    Ok(Value::Unspecified)
}

fn write_u8(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 2 {
        return Err(SchemeError::arity("'write-u8' expects 1 or 2 arguments"));
    }
    let byte = expect_byte("write-u8", &args[0])?;
    let port = optional_output_port(engine, args.get(1), "write-u8")?;
    port.borrow_mut().write_bytes(&[byte])?;
    Ok(Value::Unspecified)
}

fn write_bytevector(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.is_empty() || args.len() > 4 {
        return Err(SchemeError::arity(
            "'write-bytevector' expects 1 to 4 arguments",
        ));
    }

    let bytes = expect_bytevector("write-bytevector", &args[0])?;
    let port = optional_output_port(engine, args.get(1), "write-bytevector")?;
    let slice = {
        let bytes_ref = bytes.borrow();
        let (start, end) = parse_range(
            "write-bytevector",
            bytes_ref.len(),
            args.get(2),
            args.get(3),
        )?;
        bytes_ref[start..end].to_vec()
    };
    port.borrow_mut().write_bytes(&slice)?;
    Ok(Value::Unspecified)
}

fn flush_output_port(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() > 1 {
        return Err(SchemeError::arity(
            "'flush-output-port' expects 0 or 1 arguments",
        ));
    }
    let port = optional_output_port(engine, args.first(), "flush-output-port")?;
    port.borrow_mut().flush()?;
    Ok(Value::Unspecified)
}

fn newline(engine: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    if args.len() > 1 {
        return Err(SchemeError::arity("'newline' expects 0 or 1 arguments"));
    }
    let port = optional_output_port(engine, args.first(), "newline")?;
    port.borrow_mut().write_str("\n")?;
    Ok(Value::Unspecified)
}

fn port_predicate(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("port?", args, 1)?;
    Ok(Value::Boolean(matches!(args[0], Value::Port(_))))
}

fn input_port_predicate(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("input-port?", args, 1)?;
    let is_input = matches!(&args[0], Value::Port(port) if port.borrow().is_input());
    Ok(Value::Boolean(is_input))
}

fn output_port_predicate(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("output-port?", args, 1)?;
    let is_output = matches!(&args[0], Value::Port(port) if port.borrow().is_output());
    Ok(Value::Boolean(is_output))
}

fn textual_port_predicate(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("textual-port?", args, 1)?;
    let is_textual = matches!(&args[0], Value::Port(port) if port.borrow().is_textual());
    Ok(Value::Boolean(is_textual))
}

fn binary_port_predicate(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("binary-port?", args, 1)?;
    let is_binary = matches!(&args[0], Value::Port(port) if port.borrow().is_binary());
    Ok(Value::Boolean(is_binary))
}

fn input_port_open(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("input-port-open?", args, 1)?;
    let port = expect_input_port("input-port-open?", &args[0])?;
    let open = port.borrow().is_open();
    Ok(Value::Boolean(open))
}

fn output_port_open(_: &Engine, args: &[Value]) -> Result<Value, SchemeError> {
    expect_arity("output-port-open?", args, 1)?;
    let port = expect_output_port("output-port-open?", &args[0])?;
    let open = port.borrow().is_open();
    Ok(Value::Boolean(open))
}
