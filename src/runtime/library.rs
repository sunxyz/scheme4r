use std::collections::HashMap;

use crate::{eval::syntax::SyntaxRules, runtime::Value};

#[derive(Clone, Debug, Default)]
pub struct Library {
    bindings: HashMap<String, Value>,
    syntax_bindings: HashMap<String, SyntaxRules>,
}

impl Library {
    pub fn new(
        bindings: HashMap<String, Value>,
        syntax_bindings: HashMap<String, SyntaxRules>,
    ) -> Self {
        Self {
            bindings,
            syntax_bindings,
        }
    }

    pub fn bindings(&self) -> &HashMap<String, Value> {
        &self.bindings
    }

    pub fn syntax_bindings(&self) -> &HashMap<String, SyntaxRules> {
        &self.syntax_bindings
    }
}
