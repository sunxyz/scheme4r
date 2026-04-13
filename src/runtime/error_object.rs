use std::rc::Rc;

use crate::runtime::Value;

pub type ErrorObjectRef = Rc<ErrorObject>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorObjectKind {
    General,
    Read,
    File,
}

#[derive(Clone, Debug)]
pub struct ErrorObject {
    kind: ErrorObjectKind,
    message: String,
    irritants: Vec<Value>,
}

impl ErrorObject {
    pub fn new(
        kind: ErrorObjectKind,
        message: impl Into<String>,
        irritants: Vec<Value>,
    ) -> ErrorObjectRef {
        Rc::new(Self {
            kind,
            message: message.into(),
            irritants,
        })
    }

    pub fn kind(&self) -> &ErrorObjectKind {
        &self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn irritants(&self) -> &[Value] {
        &self.irritants
    }
}

impl PartialEq for ErrorObject {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.message == other.message
            && self.irritants.len() == other.irritants.len()
            && self
                .irritants
                .iter()
                .zip(other.irritants.iter())
                .all(|(left, right)| Value::equal(left, right))
    }
}

impl Eq for ErrorObject {}
