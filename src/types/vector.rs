use std::{fmt::Display, rc::Rc};

use super::{
    refs::{new, RefOps, Refs},
    Type,
};

#[derive(Debug, PartialEq)]
pub struct Vector {
    value: Refs<Vec<Type>>,
}

impl Vector {
    pub fn new(value: Vec<Type>) -> Vector {
        Vector { value: new(value) }
    }
    pub(in crate) fn push(&mut self, item: Type) {
        self.value.ref_write().push(item)
    }
    pub(in crate) fn push_vec(&mut self, elem: Vec<Type>) {
        self.value.ref_write().extend(elem);
    }
}

impl Clone for Vector {
    fn clone(&self) -> Self {
        Self {
            value: Rc::clone(&self.value),
        }
    }
}

impl Display for Vector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#({})",
            self.value
                .ref_read()
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}
