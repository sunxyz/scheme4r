use std::{iter::Map, rc::Rc};

use super::{refs::Refs, Symbol, Type};

#[derive(Debug)]
pub struct Record(Refs<Map<Symbol, Type>>);

impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ptr()==other.0.as_ptr()
    }
}

impl Clone for Record {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}