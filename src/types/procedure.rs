use std::{rc::Rc, fmt::Debug};

use super::{types::{Symbol, Type}, func::ApplyArgs};


pub struct Procedure {
    name: Symbol,
    value: Rc<dyn Fn(&mut ApplyArgs) -> Type>,
}

impl  Procedure{
    pub fn new(name: Symbol, value: Rc<dyn Fn(&mut ApplyArgs) -> Type>) -> Procedure{
        Procedure{
            name,
            value
        }
    }
    pub fn call(&self, args: &mut ApplyArgs) -> Type{
        let t = self.value.as_ref()(args);
        return t;
    }
    pub fn get_name(&self) -> &str{
        &self.name
    }
}

impl Clone for Procedure{
    fn clone(&self) -> Self {
        Self { name: self.name.clone(), value: Rc::clone(&self.value) }
    }
}

impl PartialEq for Procedure {
    fn eq(&self, other: &Self) -> bool {
        //TODO
        self.name == other.name
    }

}

impl Debug for Procedure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Procedure").field("name", &self.name).finish()
    }
}