use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

pub type Refs<T> = Rc<RefCell<T>>;

pub trait RefOps<T>: Clone {
    fn ref_read(&self) -> Ref<T>;
    fn ref_write(&self) -> RefMut<T>;
}

impl<T> RefOps<T> for Refs<T> {
    fn ref_read(&self) -> Ref<T> {
        self.borrow()
    }

    fn ref_write(&self) -> RefMut<T> {
        self.borrow_mut()
    }
}

pub fn new<V>(t: V) -> Refs<V> {
    Rc::new(RefCell::new(t))
}

