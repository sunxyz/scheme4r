use std::{
    cell::{Ref, RefCell, RefMut},
    fmt,
    rc::Rc,
};

use crate::{
    reader::{datum::fmt_character, Datum},
    runtime::{
        environment::EnvRef,
        error_object::ErrorObjectRef,
        pair::{PairCell, PairRef},
        parameter::ParameterRef,
        port::PortRef,
        procedure::{BuiltinFn, LambdaClause, NativeFn, Procedure, ProcedureRef},
    },
};

pub type VectorRef = Rc<RefCell<Vec<Value>>>;
pub type ByteVectorRef = Rc<RefCell<Vec<u8>>>;
pub type StringRef = Rc<RefCell<String>>;

#[derive(Clone, Debug)]
pub struct SchemeString(StringRef);

impl SchemeString {
    pub fn new(text: impl Into<String>) -> Self {
        Self(Rc::new(RefCell::new(text.into())))
    }

    pub fn borrow(&self) -> Ref<'_, String> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, String> {
        self.0.borrow_mut()
    }

    pub fn to_plain_string(&self) -> String {
        self.borrow().clone()
    }
}

impl PartialEq for SchemeString {
    fn eq(&self, other: &Self) -> bool {
        self.borrow().as_str() == other.borrow().as_str()
    }
}

impl Eq for SchemeString {}

impl PartialEq<&str> for SchemeString {
    fn eq(&self, other: &&str) -> bool {
        self.borrow().as_str() == *other
    }
}

impl PartialEq<SchemeString> for &str {
    fn eq(&self, other: &SchemeString) -> bool {
        *self == other.borrow().as_str()
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Boolean(bool),
    Number(i64),
    Character(char),
    String(SchemeString),
    Symbol(String),
    Pair(PairRef),
    Vector(VectorRef),
    ByteVector(ByteVectorRef),
    Port(PortRef),
    ErrorObject(ErrorObjectRef),
    Parameter(ParameterRef),
    Continuation(usize),
    EmptyList,
    Procedure(ProcedureRef),
    Multiple(Vec<Value>),
    EofObject,
    Unspecified,
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Self::Boolean(false))
    }

    pub fn pair(car: Value, cdr: Value) -> Self {
        Self::Pair(PairCell::new(car, cdr))
    }

    pub fn vector(items: Vec<Value>) -> Self {
        Self::Vector(Rc::new(RefCell::new(items)))
    }

    pub fn string(text: impl Into<String>) -> Self {
        Self::String(SchemeString::new(text))
    }

    pub fn bytevector(items: Vec<u8>) -> Self {
        Self::ByteVector(Rc::new(RefCell::new(items)))
    }

    pub fn port(port: PortRef) -> Self {
        Self::Port(port)
    }

    pub fn error_object(error: ErrorObjectRef) -> Self {
        Self::ErrorObject(error)
    }

    pub fn parameter(parameter: ParameterRef) -> Self {
        Self::Parameter(parameter)
    }

    pub fn list(items: Vec<Value>) -> Self {
        Self::list_with_tail(items, Self::EmptyList)
    }

    pub fn list_with_tail(items: Vec<Value>, tail: Value) -> Self {
        items
            .into_iter()
            .rev()
            .fold(tail, |cdr, car| Self::pair(car, cdr))
    }

    pub fn builtin(name: &'static str, func: BuiltinFn) -> Self {
        Self::Procedure(Procedure::builtin(name, func))
    }

    pub fn native(name: &'static str, func: NativeFn) -> Self {
        Self::Procedure(Procedure::native(name, func))
    }

    pub fn lambda(
        name: Option<String>,
        params: Vec<String>,
        rest: Option<String>,
        body: Vec<Datum>,
        env: EnvRef,
    ) -> Self {
        Self::Procedure(Procedure::lambda(name, params, rest, body, env))
    }

    pub fn case_lambda(name: Option<String>, clauses: Vec<LambdaClause>, env: EnvRef) -> Self {
        Self::Procedure(Procedure::case_lambda(name, clauses, env))
    }

    pub fn symbol(name: impl Into<String>) -> Self {
        Self::Symbol(name.into())
    }

    pub fn multiple(values: Vec<Value>) -> Self {
        Self::Multiple(values)
    }

    pub fn from_datum(datum: &Datum) -> Self {
        match datum {
            Datum::Boolean(value) => Self::Boolean(*value),
            Datum::Number(value) => Self::Number(*value),
            Datum::Character(value) => Self::Character(*value),
            Datum::String(value) => Self::string(value.clone()),
            Datum::Symbol(value) => Self::symbol(value.clone()),
            Datum::Vector(values) => Self::vector(values.iter().map(Self::from_datum).collect()),
            Datum::ByteVector(values) => Self::bytevector(values.clone()),
            Datum::EmptyList => Self::EmptyList,
            Datum::Pair(car, cdr) => Self::pair(Self::from_datum(car), Self::from_datum(cdr)),
        }
    }

    pub fn to_datum(&self) -> Result<Datum, crate::error::SchemeError> {
        match self {
            Self::Boolean(value) => Ok(Datum::Boolean(*value)),
            Self::Number(value) => Ok(Datum::Number(*value)),
            Self::Character(value) => Ok(Datum::Character(*value)),
            Self::String(value) => Ok(Datum::String(value.to_plain_string())),
            Self::Symbol(value) => Ok(Datum::Symbol(value.clone())),
            Self::Vector(values) => Ok(Datum::Vector(
                values
                    .borrow()
                    .iter()
                    .map(Value::to_datum)
                    .collect::<Result<Vec<_>, _>>()?,
            )),
            Self::ByteVector(values) => Ok(Datum::ByteVector(values.borrow().clone())),
            Self::EmptyList => Ok(Datum::EmptyList),
            Self::Pair(pair) => {
                let pair = pair.borrow();
                Ok(Datum::pair(pair.car.to_datum()?, pair.cdr.to_datum()?))
            }
            Self::Procedure(_) => Err(crate::error::SchemeError::type_error(
                "cannot convert a procedure to datum",
            )),
            Self::ErrorObject(_) => Err(crate::error::SchemeError::type_error(
                "cannot convert an error object to datum",
            )),
            Self::Parameter(_) => Err(crate::error::SchemeError::type_error(
                "cannot convert a parameter object to datum",
            )),
            Self::Multiple(_) => Err(crate::error::SchemeError::type_error(
                "cannot convert multiple values to datum",
            )),
            Self::Port(_) => Err(crate::error::SchemeError::type_error(
                "cannot convert a port to datum",
            )),
            Self::Continuation(_) => Err(crate::error::SchemeError::type_error(
                "cannot convert a continuation to datum",
            )),
            Self::EofObject => Err(crate::error::SchemeError::type_error(
                "cannot convert the eof object to datum",
            )),
            Self::Unspecified => Err(crate::error::SchemeError::type_error(
                "cannot convert an unspecified value to datum",
            )),
        }
    }

    pub fn is_proper_list(&self) -> bool {
        let mut current = self.clone();
        loop {
            match current {
                Self::EmptyList => return true,
                Self::Pair(pair) => {
                    current = pair.borrow().cdr.clone();
                }
                _ => return false,
            }
        }
    }

    pub fn to_proper_list_vec(&self) -> Option<Vec<Value>> {
        let mut items = Vec::new();
        let mut current = self.clone();

        loop {
            match current {
                Self::EmptyList => return Some(items),
                Self::Pair(pair) => {
                    let pair = pair.borrow();
                    items.push(pair.car.clone());
                    current = pair.cdr.clone();
                }
                _ => return None,
            }
        }
    }

    pub fn eqv(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Self::Boolean(left), Self::Boolean(right)) => left == right,
            (Self::Number(left), Self::Number(right)) => left == right,
            (Self::Character(left), Self::Character(right)) => left == right,
            (Self::String(left), Self::String(right)) => left == right,
            (Self::Symbol(left), Self::Symbol(right)) => left == right,
            (Self::EmptyList, Self::EmptyList) => true,
            (Self::EofObject, Self::EofObject) => true,
            (Self::Unspecified, Self::Unspecified) => true,
            (Self::Multiple(left), Self::Multiple(right)) => {
                left.len() == right.len()
                    && left
                        .iter()
                        .zip(right.iter())
                        .all(|(left, right)| Self::eqv(left, right))
            }
            (Self::Pair(left), Self::Pair(right)) => Rc::ptr_eq(left, right),
            (Self::Vector(left), Self::Vector(right)) => Rc::ptr_eq(left, right),
            (Self::ByteVector(left), Self::ByteVector(right)) => Rc::ptr_eq(left, right),
            (Self::Port(left), Self::Port(right)) => Rc::ptr_eq(left, right),
            (Self::ErrorObject(left), Self::ErrorObject(right)) => Rc::ptr_eq(left, right),
            (Self::Parameter(left), Self::Parameter(right)) => Rc::ptr_eq(left, right),
            (Self::Continuation(left), Self::Continuation(right)) => left == right,
            (Self::Procedure(left), Self::Procedure(right)) => Rc::ptr_eq(left, right),
            _ => false,
        }
    }

    pub fn equal(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Self::Pair(left), Self::Pair(right)) => {
                let left = left.borrow();
                let right = right.borrow();
                Self::equal(&left.car, &right.car) && Self::equal(&left.cdr, &right.cdr)
            }
            (Self::Vector(left), Self::Vector(right)) => {
                let left = left.borrow();
                let right = right.borrow();
                left.len() == right.len()
                    && left
                        .iter()
                        .zip(right.iter())
                        .all(|(left, right)| Self::equal(left, right))
            }
            (Self::ByteVector(left), Self::ByteVector(right)) => *left.borrow() == *right.borrow(),
            (Self::Multiple(left), Self::Multiple(right)) => {
                left.len() == right.len()
                    && left
                        .iter()
                        .zip(right.iter())
                        .all(|(left, right)| Self::equal(left, right))
            }
            _ => Self::eqv(a, b),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(value) => write!(f, "{}", if *value { "#t" } else { "#f" }),
            Self::Number(value) => write!(f, "{value}"),
            Self::Character(value) => write!(f, "{}", fmt_character(*value)),
            Self::String(value) => write!(f, "\"{}\"", value.borrow()),
            Self::Symbol(value) => write!(f, "{value}"),
            Self::Vector(values) => {
                write!(f, "#(")?;
                fmt_vector(&values.borrow(), f)?;
                write!(f, ")")
            }
            Self::ByteVector(values) => {
                write!(f, "#u8(")?;
                fmt_bytevector(&values.borrow(), f)?;
                write!(f, ")")
            }
            Self::Port(port) => write!(f, "#<{}>", port.borrow().display_name()),
            Self::ErrorObject(error) => write!(f, "#<error-object:{}>", error.message()),
            Self::Parameter(_) => write!(f, "#<parameter>"),
            Self::Continuation(_) => write!(f, "#<continuation>"),
            Self::EmptyList => write!(f, "()"),
            Self::Procedure(proc) => match proc.name() {
                Some(name) => write!(f, "#<procedure:{name}>"),
                None => write!(f, "#<procedure>"),
            },
            Self::Multiple(values) => {
                write!(f, "#<values")?;
                for value in values {
                    write!(f, " {value}")?;
                }
                write!(f, ">")
            }
            Self::EofObject => write!(f, "#<eof>"),
            Self::Unspecified => write!(f, "#<unspecified>"),
            Self::Pair(pair) => {
                write!(f, "(")?;
                fmt_pair(pair, f)?;
                write!(f, ")")
            }
        }
    }
}

fn fmt_vector(values: &[Value], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            write!(f, " ")?;
        }
        write!(f, "{value}")?;
    }
    Ok(())
}

fn fmt_bytevector(values: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            write!(f, " ")?;
        }
        write!(f, "{value}")?;
    }
    Ok(())
}

fn fmt_pair(pair: &PairRef, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut first = true;
    let mut current = Value::Pair(pair.clone());

    loop {
        match current {
            Value::Pair(pair_ref) => {
                let (car, cdr) = {
                    let pair = pair_ref.borrow();
                    (pair.car.clone(), pair.cdr.clone())
                };
                if !first {
                    write!(f, " ")?;
                }
                write!(f, "{car}")?;
                current = cdr;
                first = false;
            }
            Value::EmptyList => return Ok(()),
            other => {
                write!(f, " . {other}")?;
                return Ok(());
            }
        }
    }
}
