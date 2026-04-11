use std::{fmt, rc::Rc};

use crate::{
    error::SchemeError,
    reader::Datum,
    runtime::{environment::EnvRef, value::Value},
};

pub type BuiltinFn = fn(&[Value]) -> Result<Value, SchemeError>;

pub type ProcedureRef = Rc<Procedure>;

#[derive(Clone)]
pub enum Procedure {
    Builtin {
        name: &'static str,
        func: BuiltinFn,
    },
    Lambda {
        name: Option<String>,
        params: Vec<String>,
        body: Vec<Datum>,
        env: EnvRef,
    },
}

impl Procedure {
    pub fn builtin(name: &'static str, func: BuiltinFn) -> ProcedureRef {
        Rc::new(Self::Builtin { name, func })
    }

    pub fn lambda(
        name: Option<String>,
        params: Vec<String>,
        body: Vec<Datum>,
        env: EnvRef,
    ) -> ProcedureRef {
        Rc::new(Self::Lambda {
            name,
            params,
            body,
            env,
        })
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Builtin { name, .. } => Some(name),
            Self::Lambda { name, .. } => name.as_deref(),
        }
    }
}

impl fmt::Debug for Procedure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Builtin { name, .. } => f.debug_tuple("Builtin").field(name).finish(),
            Self::Lambda { name, params, .. } => f
                .debug_struct("Lambda")
                .field("name", name)
                .field("params", params)
                .finish(),
        }
    }
}
