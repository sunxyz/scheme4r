use std::collections::HashMap;

use crate::{
    error::SchemeError,
    eval::Engine,
    runtime::{BuiltinFn, EnvRef, Environment, Value},
};

pub type BuiltinRegistry = HashMap<String, BuiltinFn>;

pub struct Scheme {
    engine: Engine,
}

impl Scheme {
    pub fn new(builtins: BuiltinRegistry) -> Self {
        Self::with_env_and_builtins(Environment::standard(), builtins)
    }

    pub fn with_env(env: EnvRef) -> Self {
        Self {
            engine: Engine::new(env),
        }
    }

    pub fn with_env_and_builtins(env: EnvRef, builtins: BuiltinRegistry) -> Self {
        {
            let mut env_ref = env.borrow_mut();
            for (name, func) in builtins {
                env_ref.define(name.clone(), Value::builtin(name, func));
            }
        }

        Self::with_env(env)
    }

    pub fn standard() -> Self {
        Self::new(HashMap::new())
    }

    pub fn eval(&self, source: &str) -> Result<Value, SchemeError> {
        self.engine.run(source)
    }
}

impl Default for Scheme {
    fn default() -> Self {
        Self::standard()
    }
}

pub fn eval(source: &str) -> Result<Value, SchemeError> {
    Scheme::standard().eval(source)
}

pub fn interpreter(source: &str, env: EnvRef) -> Result<Value, SchemeError> {
    Scheme::with_env(env).eval(source)
}
