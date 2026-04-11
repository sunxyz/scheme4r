use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{error::SchemeError, runtime::Value};

pub type EnvRef = Rc<RefCell<Environment>>;

#[derive(Debug)]
pub struct Environment {
    parent: Option<EnvRef>,
    bindings: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> EnvRef {
        Rc::new(RefCell::new(Self {
            parent: None,
            bindings: HashMap::new(),
        }))
    }

    pub fn child(parent: EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Self {
            parent: Some(parent),
            bindings: HashMap::new(),
        }))
    }

    pub fn standard() -> EnvRef {
        let env = Self::new();
        crate::eval::builtins::install(&mut env.borrow_mut());
        env
    }

    pub fn define(&mut self, name: impl Into<String>, value: Value) {
        self.bindings.insert(name.into(), value);
    }

    pub fn lookup(&self, name: &str) -> Result<Value, SchemeError> {
        if let Some(value) = self.bindings.get(name) {
            return Ok(value.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().lookup(name);
        }

        Err(SchemeError::name(format!("undefined variable: {name}")))
    }

    pub fn set(&mut self, name: &str, value: Value) -> Result<(), SchemeError> {
        if let Some(slot) = self.bindings.get_mut(name) {
            *slot = value;
            return Ok(());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow_mut().set(name, value);
        }

        Err(SchemeError::name(format!("undefined variable: {name}")))
    }
}
