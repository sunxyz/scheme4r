use std::{error::Error, fmt};

use crate::reader::span::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Read,
    Syntax,
    Runtime,
    Type,
    Arity,
    Name,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemeError {
    pub kind: ErrorKind,
    pub message: String,
    pub span: Option<Span>,
}

impl SchemeError {
    pub fn new(kind: ErrorKind, message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            kind,
            message: message.into(),
            span,
        }
    }

    pub fn read(message: impl Into<String>, span: Option<Span>) -> Self {
        Self::new(ErrorKind::Read, message, span)
    }

    pub fn syntax(message: impl Into<String>, span: Option<Span>) -> Self {
        Self::new(ErrorKind::Syntax, message, span)
    }

    pub fn runtime(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Runtime, message, None)
    }

    pub fn type_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Type, message, None)
    }

    pub fn arity(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Arity, message, None)
    }

    pub fn name(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Name, message, None)
    }
}

impl fmt::Display for SchemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.span {
            Some(span) => write!(
                f,
                "{:?} error at {}:{}: {}",
                self.kind, span.line, span.column, self.message
            ),
            None => write!(f, "{:?} error: {}", self.kind, self.message),
        }
    }
}

impl Error for SchemeError {}
