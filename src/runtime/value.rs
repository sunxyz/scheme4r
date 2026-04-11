use std::{fmt, rc::Rc};

use crate::{
    reader::Datum,
    runtime::{
        environment::EnvRef,
        pair::{PairCell, PairRef},
        procedure::{BuiltinFn, Procedure, ProcedureRef},
    },
};

#[derive(Clone, Debug)]
pub enum Value {
    Boolean(bool),
    Number(i64),
    String(String),
    Symbol(String),
    Pair(PairRef),
    EmptyList,
    Procedure(ProcedureRef),
    Unspecified,
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Self::Boolean(false))
    }

    pub fn pair(car: Value, cdr: Value) -> Self {
        Self::Pair(PairCell::new(car, cdr))
    }

    pub fn list(items: Vec<Value>) -> Self {
        items
            .into_iter()
            .rev()
            .fold(Self::EmptyList, |cdr, car| Self::pair(car, cdr))
    }

    pub fn builtin(name: &'static str, func: BuiltinFn) -> Self {
        Self::Procedure(Procedure::builtin(name, func))
    }

    pub fn lambda(
        name: Option<String>,
        params: Vec<String>,
        body: Vec<Datum>,
        env: EnvRef,
    ) -> Self {
        Self::Procedure(Procedure::lambda(name, params, body, env))
    }

    pub fn symbol(name: impl Into<String>) -> Self {
        Self::Symbol(name.into())
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

    pub fn eqv(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Self::Boolean(left), Self::Boolean(right)) => left == right,
            (Self::Number(left), Self::Number(right)) => left == right,
            (Self::String(left), Self::String(right)) => left == right,
            (Self::Symbol(left), Self::Symbol(right)) => left == right,
            (Self::EmptyList, Self::EmptyList) => true,
            (Self::Unspecified, Self::Unspecified) => true,
            (Self::Pair(left), Self::Pair(right)) => Rc::ptr_eq(left, right),
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
            _ => Self::eqv(a, b),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(value) => write!(f, "{}", if *value { "#t" } else { "#f" }),
            Self::Number(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "\"{}\"", value),
            Self::Symbol(value) => write!(f, "{value}"),
            Self::EmptyList => write!(f, "()"),
            Self::Procedure(proc) => match proc.name() {
                Some(name) => write!(f, "#<procedure:{name}>"),
                None => write!(f, "#<procedure>"),
            },
            Self::Unspecified => write!(f, "#<unspecified>"),
            Self::Pair(pair) => {
                write!(f, "(")?;
                fmt_pair(pair, f)?;
                write!(f, ")")
            }
        }
    }
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
