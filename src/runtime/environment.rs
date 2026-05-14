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
        let bindings = env.borrow().bindings.clone();
        let mut base_bindings = bindings.clone();
        for name in NON_BASE_LIBRARY_NAMES {
            base_bindings.remove(*name);
        }
        let base_library = Library::new(base_bindings);
        let read_library = Library::new(select_bindings(&bindings, STANDARD_READ_LIBRARY_NAMES));
        let write_library = Library::new(select_bindings(&bindings, STANDARD_WRITE_LIBRARY_NAMES));
        let eval_library = Library::new(select_bindings(&bindings, STANDARD_EVAL_LIBRARY_NAMES));
        {
            let mut env_mut = env.borrow_mut();
            env_mut.define_library("scheme base", base_library);
            env_mut.define_library("scheme read", read_library);
            env_mut.define_library("scheme write", write_library);
            env_mut.define_library("scheme eval", eval_library);
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

fn select_bindings(
    bindings: &HashMap<String, Value>,
    names: &[&str],
) -> HashMap<String, Value> {
    let mut selected = HashMap::new();
    for name in names {
        if let Some(value) = bindings.get(*name) {
            selected.insert((*name).to_string(), value.clone());
        }
    }
    selected
}

const STANDARD_READ_LIBRARY_NAMES: &[&str] = &[
    "read",
    "read-char",
    "read-line",
    "peek-char",
    "char-ready?",
    "read-string",
    "read-u8",
    "peek-u8",
    "u8-ready?",
    "read-bytevector",
    "read-bytevector!",
    "eof-object?",
];

const STANDARD_WRITE_LIBRARY_NAMES: &[&str] = &[
    "display",
    "write",
    "write-shared",
    "write-simple",
    "write-char",
    "write-string",
    "write-u8",
    "write-bytevector",
    "flush-output-port",
    "newline",
];

const STANDARD_EVAL_LIBRARY_NAMES: &[&str] = &["eval", "environment"];

const NON_BASE_LIBRARY_NAMES: &[&str] = &[
    "read",
    "read-char",
    "read-line",
    "peek-char",
    "char-ready?",
    "read-string",
    "read-u8",
    "peek-u8",
    "u8-ready?",
    "read-bytevector",
    "read-bytevector!",
    "display",
    "write",
    "write-shared",
    "write-simple",
    "write-char",
    "write-string",
    "write-u8",
    "write-bytevector",
    "flush-output-port",
    "newline",
    "eval",
    "environment",
    "load",
    "port?",
    "input-port?",
    "output-port?",
    "textual-port?",
    "binary-port?",
    "input-port-open?",
    "output-port-open?",
    "current-input-port",
    "current-output-port",
    "current-error-port",
    "open-input-string",
    "open-output-string",
    "get-output-string",
    "open-input-bytevector",
    "open-output-bytevector",
    "get-output-bytevector",
    "open-input-file",
    "open-output-file",
    "close-port",
    "close-input-port",
    "close-output-port",
    "call-with-port",
];
