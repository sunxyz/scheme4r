use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    error::SchemeError,
    eval::syntax::SyntaxRules,
    runtime::{library::Library, Value},
};

pub type EnvRef = Rc<RefCell<Environment>>;

#[derive(Debug)]
pub struct Environment {
    parent: Option<EnvRef>,
    bindings: HashMap<String, Value>,
    syntax_bindings: HashMap<String, SyntaxRules>,
    library_registry: Rc<RefCell<HashMap<String, Library>>>,
}

impl Environment {
    pub fn new() -> EnvRef {
        Rc::new(RefCell::new(Self {
            parent: None,
            bindings: HashMap::new(),
            syntax_bindings: HashMap::new(),
            library_registry: Rc::new(RefCell::new(HashMap::new())),
        }))
    }

    pub fn child(parent: EnvRef) -> EnvRef {
        let library_registry = parent.borrow().library_registry.clone();
        Rc::new(RefCell::new(Self {
            parent: Some(parent),
            bindings: HashMap::new(),
            syntax_bindings: HashMap::new(),
            library_registry,
        }))
    }

    pub fn isolated_with_registry(source: EnvRef) -> EnvRef {
        let library_registry = source.borrow().library_registry.clone();
        Rc::new(RefCell::new(Self {
            parent: None,
            bindings: HashMap::new(),
            syntax_bindings: HashMap::new(),
            library_registry,
        }))
    }

    pub fn standard() -> EnvRef {
        let env = Self::new();
        crate::eval::builtins::install(&mut env.borrow_mut());
        let base_library = Library::new(env.borrow().bindings.clone());
        {
            let mut env_mut = env.borrow_mut();
            env_mut.define_library("scheme base", base_library.clone());
            env_mut.define_library("scheme write", base_library.clone());
            env_mut.define_library("scheme read", base_library);
        }
        env
    }

    pub fn define(&mut self, name: impl Into<String>, value: Value) {
        self.bindings.insert(name.into(), value);
    }

    pub fn define_syntax(&mut self, name: impl Into<String>, transformer: SyntaxRules) {
        self.syntax_bindings.insert(name.into(), transformer);
    }

    pub fn define_library(&mut self, name: impl Into<String>, library: Library) {
        self.library_registry
            .borrow_mut()
            .insert(name.into(), library);
    }

    pub fn lookup_library(&self, name: &str) -> Option<Library> {
        self.library_registry.borrow().get(name).cloned()
    }

    pub fn import_bindings(&mut self, bindings: &HashMap<String, Value>) {
        for (name, value) in bindings {
            self.bindings.insert(name.clone(), value.clone());
        }
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

    pub fn lookup_syntax(&self, name: &str) -> Option<SyntaxRules> {
        if let Some(transformer) = self.syntax_bindings.get(name) {
            return Some(transformer.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().lookup_syntax(name);
        }

        None
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
