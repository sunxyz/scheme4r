use std::{error::Error, fmt};

use crate::{reader::span::Span, runtime::Value};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Read,
    Syntax,
    Runtime,
    Type,
    Arity,
    Name,
    Io,
}

#[derive(Clone, Debug)]
pub enum ControlSignal {
    ContinuationJump { token: usize, value: Value },
    Raised { object: Value, continuable: bool },
}

#[derive(Clone, Debug)]
pub struct SchemeError {
    pub kind: ErrorKind,
    pub message: String,
    pub span: Option<Span>,
    pub signal: Option<ControlSignal>,
}

impl SchemeError {
    pub fn new(kind: ErrorKind, message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            kind,
            message: message.into(),
            span,
            signal: None,
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

    pub fn io(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Io, message, None)
    }

    pub fn raised(object: Value, continuable: bool) -> Self {
        Self {
            kind: ErrorKind::Runtime,
            message: if continuable {
                "continuable exception raised".to_string()
            } else {
                "exception raised".to_string()
            },
            span: None,
            signal: Some(ControlSignal::Raised {
                object,
                continuable,
            }),
        }
    }

    pub fn continuation_jump(token: usize, value: Value) -> Self {
        Self {
            kind: ErrorKind::Runtime,
            message: "continuation jump".to_string(),
            span: None,
            signal: Some(ControlSignal::ContinuationJump { token, value }),
        }
    }

    pub fn as_raised(&self) -> Option<(&Value, bool)> {
        match &self.signal {
            Some(ControlSignal::Raised {
                object,
                continuable,
            }) => Some((object, *continuable)),
            _ => None,
        }
    }

    pub fn as_continuation_jump(&self) -> Option<(usize, &Value)> {
        match &self.signal {
            Some(ControlSignal::ContinuationJump { token, value }) => Some((*token, value)),
            _ => None,
        }
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

impl PartialEq for ControlSignal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::ContinuationJump {
                    token: left_token,
                    value: left_value,
                },
                Self::ContinuationJump {
                    token: right_token,
                    value: right_value,
                },
            ) => left_token == right_token && Value::equal(left_value, right_value),
            (
                Self::Raised {
                    object: left_object,
                    continuable: left_continuable,
                },
                Self::Raised {
                    object: right_object,
                    continuable: right_continuable,
                },
            ) => left_continuable == right_continuable && Value::equal(left_object, right_object),
            _ => false,
        }
    }
}

impl Eq for ControlSignal {}

impl PartialEq for SchemeError {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.message == other.message
            && self.span == other.span
            && self.signal == other.signal
    }
}

impl Eq for SchemeError {}
