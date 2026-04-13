use std::{
    cell::RefCell,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::runtime::Value;

pub type ParameterRef = Rc<ParameterObject>;

static NEXT_PARAMETER_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Debug)]
pub struct ParameterObject {
    id: usize,
    cell: Rc<RefCell<Value>>,
    converter: Option<Value>,
}

impl ParameterObject {
    pub fn new(initial: Value, converter: Option<Value>) -> ParameterRef {
        Rc::new(Self {
            id: NEXT_PARAMETER_ID.fetch_add(1, Ordering::Relaxed),
            cell: Rc::new(RefCell::new(initial)),
            converter,
        })
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn cell(&self) -> Rc<RefCell<Value>> {
        self.cell.clone()
    }

    pub fn converter(&self) -> Option<Value> {
        self.converter.clone()
    }
}
