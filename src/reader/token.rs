use crate::reader::span::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    LParen,
    RParen,
    Dot,
    Quote,
    Quasiquote,
    Unquote,
    UnquoteSplicing,
    VectorStart,
    ByteVectorStart,
    Boolean(bool),
    Number(i64),
    Character(char),
    String(String),
    Symbol(String),
    Eof,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub start: usize,
    pub end: usize,
}

impl Token {
    pub const fn new(kind: TokenKind, span: Span, start: usize, end: usize) -> Self {
        Self {
            kind,
            span,
            start,
            end,
        }
    }
}
