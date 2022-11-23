use std::cell::RefMut;
use std::fmt::{Display, Formatter};

use super::refs::{new, RefOps, Refs};
use super::types::Type;

#[derive(Debug, PartialEq)]
pub enum ListType {
    SUB,
    EXPR,
    QUOTE,
}
#[derive(Debug)]
pub struct List(Refs<Vec<Type>>, ListType);

impl List {
    pub fn new() -> Self {
        List(new(Vec::new()), ListType::EXPR)
    }

    pub fn of(elem_s: Vec<Type>) -> Self {
        List(new(elem_s), ListType::EXPR)
    }

    pub(crate) fn of_quote(elem_s: Vec<Type>) -> Self {
        List(new(elem_s), ListType::QUOTE)
    }

    pub fn car(&self) -> Type {
        self.0.ref_read().get(0).unwrap().clone()
    }

    pub fn cdr(&self) -> List {
        let t = self.0.ref_read()[1..].to_vec();
        List(new(t), ListType::SUB)
    }

    pub fn is_nil(&self) -> bool {
        self.0.ref_read().len() == 0
    }

    pub fn is_expr(&self) -> bool {
        if let ListType::SUB = self.1  {
            false
        } else {
            true
        }
    }

    pub fn is_sub(&self) -> bool {
        if let ListType::SUB = self.1 {
            true
        } else {
            false
        }
    }

    pub(crate) fn is_quote(&self) -> bool {
        if let ListType::QUOTE = self.1 {
            true
        } else {
            false
        }
    }

    pub(crate) fn get_data(&mut self) -> RefMut<Vec<Type>>{
         self.0.ref_write()
    }

    pub fn push(&mut self, elem: Type) {
        self.0.ref_write().push(elem);
    }

    pub fn push_vec(&mut self, elem: Vec<Type>) {
        self.0.ref_write().extend(elem);
    }

    pub fn push_all(&mut self, elem: List) {
        self.0.ref_write().extend(elem.data());
    }

    pub fn len(&self) -> usize {
        self.0.ref_read().len()
    }

    pub fn data(&self) -> Vec<Type> {
        self.0.ref_read().clone()
    }
}

impl Clone for List {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl PartialEq for List {
    fn eq(&self, other: &Self) -> bool {
        if self.1 != other.1 {
            return false;
        }
        if self.0.ref_read().len() != other.0.ref_read().len() {
            return false;
        }
        for i in 0..self.0.ref_read().len() {
            if self.0.ref_read()[i] != other.0.ref_read()[i] {
                return false;
            }
        }
        true
    }
}

impl Display for List {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        if self.is_quote() && *self.0.ref_read().get(0).unwrap()==Type::symbol("quote") {
            write!(
                f,
                "'{}",
                self.0
                    .ref_read()
                    .iter().skip(1)
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            )
        }else{
            write!(
                f,
                "({})",
                self.0
                    .ref_read()
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            )
        }
        
       
    }
}

impl Clone for ListType {
    fn clone(&self) -> Self {
        match self {
            Self::SUB => Self::SUB,
            Self::EXPR => Self::EXPR,
            Self::QUOTE=>Self::QUOTE,
        }
    }
}
