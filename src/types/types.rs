use super::{
    func::ApplyArgs,
    refs::{new, RefOps, Refs},
};
use std::{fmt::Display, rc::Rc};

pub use super::{list::List, procedure::Procedure, record::Record, vector::Vector, u8vector::U8Vector};

pub type Number = isize;
pub type Boolean = bool;
pub type Pair = Refs<(Type, Type)>;
pub type SExpr = List;
pub type Symbol = String;
pub type Character = char;
pub type Strings = Refs<String>;
pub type ByteVector = U8Vector;
pub type Port = ();
pub type  Quote = Box<Type>;
#[derive(Debug, PartialEq)]
pub enum Type {
    Numbers(Number),
    Booleans(Boolean),
    Pairs(Pair),
    Lists(SExpr),
    Symbols(Symbol),
    Characters(Character),
    Strings(Strings),
    Vectors(Vector),
    ByteVectors(ByteVector),
    Procedures(Procedure),
    Records(Record),
    Ports(Port),
    Nil,
    Quotes(Quote),
    Error(String)
}

impl Clone for Type {
    fn clone(&self) -> Self {
        match self {
            Self::Numbers(arg0) => Self::Numbers(arg0.clone()),
            Self::Booleans(arg0) => Self::Booleans(arg0.clone()),
            Self::Pairs(arg0) => Self::Pairs(arg0.clone()),
            Self::Lists(arg0) => Self::Lists(arg0.clone()),
            Self::Symbols(arg0) => Self::Symbols(arg0.clone()),
            Self::Characters(arg0) => Self::Characters(arg0.clone()),
            Self::Strings(arg0) => Self::Strings(arg0.clone()),
            Self::Vectors(arg0) => Self::Vectors(arg0.clone()),
            Self::ByteVectors(arg0) => Self::ByteVectors(arg0.clone()),
            Self::Procedures(arg0) => Self::Procedures(arg0.clone()),
            Self::Records(arg0) => Self::Records(arg0.clone()),
            Self::Ports(arg0) => Self::Ports(arg0.clone()),
            Self::Nil => Self::Nil,
            Self::Quotes(t)=>Self::Quotes(t.clone()),
            Self::Error(e)=>Self::Error(e.clone())
        }
    }
}

impl Type {
    pub fn integer_of(n: isize) -> Type {
        Type::Numbers(n)
    }
    pub fn float_of(n: f64) -> Type {
        Type::Numbers(n as isize)
    }
    pub fn paris_of(car: Type, cdr: Type) -> Type {
        Type::Pairs(new((car, cdr)))
    }
    pub fn expr_of(elem_s: Vec<Type>) -> Type {
        Type::Lists(List::of(elem_s))
    }
    pub fn character_of(data: Character) -> Type {
        Type::Characters(data)
    }
    pub fn symbol_of(data: Symbol) -> Type {
        Type::Symbols(data)
    }
    pub fn symbol(data: &str) -> Type {
        Type::Symbols(data.to_string())
    }
    pub fn string_of(data: String) -> Type {
        Type::Strings(new(data))
    }
    pub fn vector_of(vec: Vec<Type>) -> Type {
        Type::Vectors(Vector::new(vec))
    }
    pub fn u8vector_of(vec: Vec<u8>) -> Type {
        Type::ByteVectors(ByteVector::new(vec))
    }
    pub fn error(data: &str) -> Type {
        Type::Error(data.to_string())
    }
    pub fn quote(data: Type) -> Type {
        Type::Quotes(Box::new(data))
    }
    pub fn procedure_of(name: &str, f: fn(&mut ApplyArgs) -> Type) -> Type {
        let name = name.to_owned();
        Type::Procedures(Procedure::new(name, Rc::new(f)))
    }
    pub fn procedure_of_rc(name: &str, f: Rc<dyn Fn(&mut ApplyArgs) -> Type>) -> Type {
        Type::Procedures(Procedure::new(name.to_owned(), f))
    }
}
impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Numbers(n) => write!(f, "{}", n),
            Type::Booleans(n) => write!(f, "{}", n),
            Type::Pairs(n) => {
                let n = n.borrow();
                write!(f, "({},{})", n.0, n.1)
            }
            Type::Lists(n) => write!(f, "{}", n),
            Type::Symbols(n) => write!(f, "{}", n),
            Type::Characters(n) => write!(f, "{}", n),
            Type::Strings(n) => write!(f, "{}", n.ref_read()),
            Type::Vectors(n) => write!(f, "{}", n),
            Type::ByteVectors(n) => write!(f, "{}", n),
            Type::Procedures(_) => write!(f, "<procedure>"),
            Type::Records(_) => write!(f, "<record>"),
            Type::Ports(_) => write!(f, "<port>",),
            Type::Nil => write!(f, "nil"),
            Type::Quotes(t)=>write!(f, "'{}", t),
            Type::Error(e) => write!(f, "<error:{}>",e),
        }
    }
}
