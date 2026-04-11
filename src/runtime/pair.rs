use std::{cell::RefCell, rc::Rc};

use crate::runtime::value::Value;

#[derive(Clone, Debug)]
pub struct PairCell {
    pub car: Value,
    pub cdr: Value,
}

pub type PairRef = Rc<RefCell<PairCell>>;

impl PairCell {
    pub fn new(car: Value, cdr: Value) -> PairRef {
        Rc::new(RefCell::new(Self { car, cdr }))
    }
}
