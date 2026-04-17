use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::runtime::Value;

pub type DictMap = BTreeMap<DictKey, Value>;
pub type DictRef = Rc<RefCell<DictMap>>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DictKey {
    Boolean(bool),
    Number(i64),
    Character(char),
    String(String),
    Symbol(String),
    EmptyList,
}

impl DictKey {
    pub fn try_from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Boolean(v) => Some(Self::Boolean(*v)),
            Value::Number(v) => Some(Self::Number(*v)),
            Value::Character(v) => Some(Self::Character(*v)),
            Value::String(v) => Some(Self::String(v.to_plain_string())),
            Value::Symbol(v) => Some(Self::Symbol(v.clone())),
            Value::EmptyList => Some(Self::EmptyList),
            _ => None,
        }
    }

    pub fn to_value(&self) -> Value {
        match self {
            Self::Boolean(v) => Value::Boolean(*v),
            Self::Number(v) => Value::Number(*v),
            Self::Character(v) => Value::Character(*v),
            Self::String(v) => Value::string(v.clone()),
            Self::Symbol(v) => Value::symbol(v.clone()),
            Self::EmptyList => Value::EmptyList,
        }
    }
}
