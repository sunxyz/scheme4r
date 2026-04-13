use crate::{
    error::SchemeError,
    eval::Engine,
    runtime::{EnvRef, Environment, Value},
};

pub struct Scheme {
    engine: Engine,
}

impl Scheme {
    pub fn new(env: EnvRef) -> Self {
        Self {
            engine: Engine::new(env),
        }
    }

    pub fn standard() -> Self {
        Self::new(Environment::standard())
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
    Scheme::new(env).eval(source)
}
