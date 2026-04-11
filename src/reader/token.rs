use crate::reader::span::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    LParen,
    RParen,
    Dot,
    Quote,
    Boolean(bool),
    Number(i64),
    String(String),
    Symbol(String),
    Eof,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}
