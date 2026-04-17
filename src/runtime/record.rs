use std::{cell::RefCell, rc::Rc};

use crate::runtime::Value;

pub type RecordTypeRef = Rc<RecordType>;
pub type RecordRef = Rc<RefCell<RecordInstance>>;

#[derive(Clone, Debug)]
pub struct RecordFieldSpec {
    name: String,
    mutable: bool,
}

impl RecordFieldSpec {
    pub fn new(name: impl Into<String>, mutable: bool) -> Self {
        Self {
            name: name.into(),
            mutable,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn is_mutable(&self) -> bool {
        self.mutable
    }
}

#[derive(Clone, Debug)]
pub struct RecordType {
    name: String,
    fields: Vec<RecordFieldSpec>,
}

impl RecordType {
    pub fn new(name: impl Into<String>, fields: Vec<RecordFieldSpec>) -> RecordTypeRef {
        Rc::new(Self {
            name: name.into(),
            fields,
        })
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    pub fn field_name(&self, index: usize) -> Option<&str> {
        self.fields.get(index).map(RecordFieldSpec::name)
    }

    pub fn field_mutable(&self, index: usize) -> Option<bool> {
        self.fields.get(index).map(RecordFieldSpec::is_mutable)
    }
}

#[derive(Clone, Debug)]
pub struct RecordInstance {
    record_type: RecordTypeRef,
    fields: Vec<Value>,
}

impl RecordInstance {
    pub fn new(record_type: RecordTypeRef, fields: Vec<Value>) -> RecordRef {
        Rc::new(RefCell::new(Self {
            record_type,
            fields,
        }))
    }

    pub fn record_type(&self) -> RecordTypeRef {
        self.record_type.clone()
    }

    pub fn field(&self, index: usize) -> Option<&Value> {
        self.fields.get(index)
    }

    pub fn set_field(&mut self, index: usize, value: Value) -> bool {
        if let Some(slot) = self.fields.get_mut(index) {
            *slot = value;
            true
        } else {
            false
        }
    }
}
