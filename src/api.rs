use crate::{
    error::SchemeError,
    eval::Engine,
    runtime::{EnvRef, Environment, Value},
};

pub fn eval(source: &str) -> Result<Value, SchemeError> {
    interpreter(source, Environment::standard())
}

pub fn interpreter(source: &str, env: EnvRef) -> Result<Value, SchemeError> {
    Engine::new(env).run(source)
}
