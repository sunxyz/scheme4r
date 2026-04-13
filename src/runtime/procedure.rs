use std::{fmt, rc::Rc};

use crate::{
    error::SchemeError,
    eval::Engine,
    reader::Datum,
    runtime::{environment::EnvRef, value::Value},
};

pub type BuiltinFn = fn(&Engine, &[Value]) -> Result<Value, SchemeError>;
pub type NativeFn = fn(&Engine, EnvRef, &[Value]) -> Result<Value, SchemeError>;

pub type ProcedureRef = Rc<Procedure>;

#[derive(Clone, Debug)]
pub struct LambdaClause {
    pub params: Vec<String>,
    pub rest: Option<String>,
    pub body: Vec<Datum>,
}

#[derive(Clone)]
pub enum Procedure {
    Builtin {
        name: String,
        func: BuiltinFn,
    },
    Native {
        name: String,
        func: NativeFn,
    },
    Lambda {
        name: Option<String>,
        params: Vec<String>,
        rest: Option<String>,
        body: Vec<Datum>,
        env: EnvRef,
    },
    CaseLambda {
        name: Option<String>,
        clauses: Vec<LambdaClause>,
        env: EnvRef,
    },
}

impl Procedure {
    pub fn builtin(name: impl Into<String>, func: BuiltinFn) -> ProcedureRef {
        Rc::new(Self::Builtin {
            name: name.into(),
            func,
        })
    }

    pub fn native(name: impl Into<String>, func: NativeFn) -> ProcedureRef {
        Rc::new(Self::Native {
            name: name.into(),
            func,
        })
    }

    pub fn lambda(
        name: Option<String>,
        params: Vec<String>,
        rest: Option<String>,
        body: Vec<Datum>,
        env: EnvRef,
    ) -> ProcedureRef {
        Rc::new(Self::Lambda {
            name,
            params,
            rest,
            body,
            env,
        })
    }

    pub fn case_lambda(
        name: Option<String>,
        clauses: Vec<LambdaClause>,
        env: EnvRef,
    ) -> ProcedureRef {
        Rc::new(Self::CaseLambda { name, clauses, env })
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Builtin { name, .. } => Some(name.as_str()),
            Self::Native { name, .. } => Some(name.as_str()),
            Self::Lambda { name, .. } => name.as_deref(),
            Self::CaseLambda { name, .. } => name.as_deref(),
        }
    }
}

impl fmt::Debug for Procedure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Builtin { name, .. } => f.debug_tuple("Builtin").field(name).finish(),
            Self::Native { name, .. } => f.debug_tuple("Native").field(name).finish(),
            Self::Lambda {
                name, params, rest, ..
            } => f
                .debug_struct("Lambda")
                .field("name", name)
                .field("params", params)
                .field("rest", rest)
                .finish(),
            Self::CaseLambda { name, clauses, .. } => f
                .debug_struct("CaseLambda")
                .field("name", name)
                .field("clauses", clauses)
                .finish(),
        }
    }
}
