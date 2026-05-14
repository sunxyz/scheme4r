use std::{cell::RefCell, convert::TryFrom, rc::Rc};

use crate::{
    error::SchemeError,
    eval::Engine,
    runtime::{environment::Environment, pair::PairCell, DictRef, PortRef, SchemeString, Value},
};

mod booleans;
mod bytevectors;
mod chars;
mod control;
mod dicts;
mod equivalence;
mod lists;
mod numbers;
mod ports;
mod procedures;
mod strings;
mod symbols;
mod vectors;

pub(crate) fn install(env: &mut Environment) {
    booleans::install(env);
    numbers::install(env);
    chars::install(env);
    strings::install(env);
    symbols::install(env);
    lists::install(env);
    vectors::install(env);
    bytevectors::install(env);
    equivalence::install(env);
    dicts::install(env);
    procedures::install(env);
    control::install(env);
    ports::install(env);
}

pub(super) fn define_builtin(
    env: &mut Environment,
    name: &'static str,
    func: fn(&Engine, &[Value]) -> Result<Value, SchemeError>,
) {
    env.define(name, Value::builtin(name, func));
}

pub(super) fn expect_arity(name: &str, args: &[Value], expected: usize) -> Result<(), SchemeError> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(SchemeError::arity(format!(
            "'{name}' expects {expected} arguments, got {}",
            args.len()
        )))
    }
}

pub(super) fn expect_number(name: &str, value: &Value) -> Result<i64, SchemeError> {
    match value {
        Value::Number(number) => Ok(*number),
        other => Err(SchemeError::type_error(format!(
            "'{name}' expected a number, got {other}"
        ))),
    }
}

pub(super) fn expect_byte(name: &str, value: &Value) -> Result<u8, SchemeError> {
    let number = expect_number(name, value)?;
    u8::try_from(number)
        .map_err(|_| SchemeError::type_error(format!("'{name}' expected a byte value")))
}

pub(super) fn expect_index(name: &str, value: &Value) -> Result<usize, SchemeError> {
    let number = expect_number(name, value)?;
    usize::try_from(number)
        .map_err(|_| SchemeError::type_error(format!("'{name}' expected a non-negative index")))
}

pub(super) fn expect_char(name: &str, value: &Value) -> Result<char, SchemeError> {
    match value {
        Value::Character(ch) => Ok(*ch),
        other => Err(SchemeError::type_error(format!(
            "'{name}' expected a character, got {other}"
        ))),
    }
}

pub(super) fn expect_port(name: &str, value: &Value) -> Result<PortRef, SchemeError> {
    match value {
        Value::Port(port) => Ok(port.clone()),
        other => Err(SchemeError::type_error(format!(
            "'{name}' expected a port, got {other}"
        ))),
    }
}

pub(super) fn expect_input_port(name: &str, value: &Value) -> Result<PortRef, SchemeError> {
    let port = expect_port(name, value)?;
    if port.borrow().is_input() {
        Ok(port)
    } else {
        Err(SchemeError::type_error(format!(
            "'{name}' expected an input port"
        )))
    }
}

pub(super) fn expect_output_port(name: &str, value: &Value) -> Result<PortRef, SchemeError> {
    let port = expect_port(name, value)?;
    if port.borrow().is_output() {
        Ok(port)
    } else {
        Err(SchemeError::type_error(format!(
            "'{name}' expected an output port"
        )))
    }
}

pub(super) fn expect_string_ref(name: &str, value: &Value) -> Result<SchemeString, SchemeError> {
    match value {
        Value::String(text) => Ok(text.clone()),
        other => Err(SchemeError::type_error(format!(
            "'{name}' expected a string, got {other}"
        ))),
    }
}

pub(super) fn expect_string(name: &str, value: &Value) -> Result<String, SchemeError> {
    Ok(expect_string_ref(name, value)?.to_plain_string())
}

pub(super) fn expect_symbol<'a>(name: &str, value: &'a Value) -> Result<&'a str, SchemeError> {
    match value {
        Value::Symbol(symbol) => Ok(symbol.as_str()),
        other => Err(SchemeError::type_error(format!(
            "'{name}' expected a symbol, got {other}"
        ))),
    }
}

pub(super) fn expect_list(name: &str, value: &Value) -> Result<Vec<Value>, SchemeError> {
    value.to_proper_list_vec().ok_or_else(|| {
        SchemeError::type_error(format!("'{name}' expected a proper list, got {value}"))
    })
}

pub(super) fn expect_vector(
    name: &str,
    value: &Value,
) -> Result<Rc<RefCell<Vec<Value>>>, SchemeError> {
    match value {
        Value::Vector(vector) => Ok(vector.clone()),
        other => Err(SchemeError::type_error(format!(
            "'{name}' expected a vector, got {other}"
        ))),
    }
}

pub(super) fn expect_bytevector(
    name: &str,
    value: &Value,
) -> Result<Rc<RefCell<Vec<u8>>>, SchemeError> {
    match value {
        Value::ByteVector(bytes) => Ok(bytes.clone()),
        other => Err(SchemeError::type_error(format!(
            "'{name}' expected a bytevector, got {other}"
        ))),
    }
}

pub(super) fn expect_pair(name: &str, value: &Value) -> Result<Rc<RefCell<PairCell>>, SchemeError> {
    match value {
        Value::Pair(pair) => Ok(pair.clone()),
        other => Err(SchemeError::type_error(format!(
            "'{name}' expected a pair, got {other}"
        ))),
    }
}

pub(super) fn expect_dict(name: &str, value: &Value) -> Result<DictRef, SchemeError> {
    match value {
        Value::Dict(dict) => Ok(dict.clone()),
        other => Err(SchemeError::type_error(format!(
            "'{name}' expected a dict, got {other}"
        ))),
    }
}

pub(super) fn optional_output_port(
    engine: &Engine,
    port_arg: Option<&Value>,
    name: &str,
) -> Result<PortRef, SchemeError> {
    match port_arg {
        Some(value) => expect_output_port(name, value),
        None => Ok(engine.current_output_port()),
    }
}

pub(super) fn optional_input_port(
    engine: &Engine,
    port_arg: Option<&Value>,
    name: &str,
) -> Result<PortRef, SchemeError> {
    match port_arg {
        Some(value) => expect_input_port(name, value),
        None => Ok(engine.current_input_port()),
    }
}

pub(super) fn parse_range(
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
        return Err(SchemeError::runtime(format!(
            "'{name}' range out of bounds"
        )));
    }

    Ok((start, end))
}
