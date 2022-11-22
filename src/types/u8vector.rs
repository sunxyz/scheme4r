use std::{fmt::Display, rc::Rc};

use super::{
    refs::{new, RefOps, Refs},
};

#[derive(Debug, PartialEq)]
pub struct U8Vector {
     value: Refs<Vec<u8>>,
}

impl U8Vector {
    pub fn new(value: Vec<u8>) -> U8Vector {
        U8Vector { value: new(value) }
    }
    pub(in crate) fn push(&mut self, item: u8) {
        self.value.ref_write().push(item)
    }
    pub(in crate) fn push_vec(&mut self, elem: Vec<u8>) {
        self.value.ref_write().extend(elem);
    }
    pub fn len(&self)->usize {
        self.value.ref_read().len()
    }
}

impl Clone for U8Vector {
    fn clone(&self) -> Self {
        Self {
            value: Rc::clone(&self.value),
        }
    }
}

impl Display for U8Vector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#u8({})",
            self.value
                .ref_read()
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}
