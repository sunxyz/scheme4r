use std::collections::HashMap;

use crate::runtime::Value;

#[derive(Clone, Debug, Default)]
pub struct Library {
    bindings: HashMap<String, Value>,
}

impl Library {
    pub fn new(bindings: HashMap<String, Value>) -> Self {
        Self { bindings }
    }

    pub fn bindings(&self) -> &HashMap<String, Value> {
        &self.bindings
    }
}
