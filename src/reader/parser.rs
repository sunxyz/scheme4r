use crate::{
    error::SchemeError,
    reader::{
        datum::Datum,
        lexer::Lexer,
        token::{Token, TokenKind},
    },
};

pub struct Reader<'a> {
    source: &'a str,
}

impl<'a> Reader<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn read_all(&self) -> Result<Vec<Datum>, SchemeError> {
        Parser::new(Lexer::new(self.source)).parse_all()
    }

    pub fn read_one(&self) -> Result<Option<(Datum, usize)>, SchemeError> {
        Parser::new(Lexer::new(self.source)).parse_one()
    }
}

struct Parser<'a> {
    lexer: Lexer<'a>,
    tokens: Vec<Token>,
    current: usize,
}

impl<'a> Parser<'a> {
    fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer,
            tokens: Vec::new(),
            current: 0,
        }
    }

    fn parse_all(&mut self) -> Result<Vec<Datum>, SchemeError> {
        let mut forms = Vec::new();
        while !self.is_at_end()? {
            forms.push(self.expression()?);
        }
        Ok(forms)
    }

    fn parse_one(&mut self) -> Result<Option<(Datum, usize)>, SchemeError> {
        if self.is_at_end()? {
            return Ok(None);
        }

        let expr = self.expression()?;
        let end = self.previous()?.end;
        Ok(Some((expr, end)))
    }

    fn expression(&mut self) -> Result<Datum, SchemeError> {
        let token = self.advance()?.clone();
        match token.kind {
            TokenKind::Boolean(value) => Ok(Datum::Boolean(value)),
            TokenKind::Number(value) => Ok(Datum::Number(value)),
            TokenKind::Character(value) => Ok(Datum::Character(value)),
            TokenKind::String(value) => Ok(Datum::String(value)),
            TokenKind::Symbol(value) => Ok(Datum::Symbol(value)),
            TokenKind::Quote => {
                let quoted = self.expression()?;
                Ok(Datum::list(vec![Datum::symbol("quote"), quoted]))
            }
            TokenKind::Quasiquote => {
                let quoted = self.expression()?;
                Ok(Datum::list(vec![Datum::symbol("quasiquote"), quoted]))
            }
            TokenKind::Unquote => {
                let quoted = self.expression()?;
                Ok(Datum::list(vec![Datum::symbol("unquote"), quoted]))
            }
            TokenKind::UnquoteSplicing => {
                let quoted = self.expression()?;
                Ok(Datum::list(vec![Datum::symbol("unquote-splicing"), quoted]))
            }
            TokenKind::LParen => self.list(token),
            TokenKind::VectorStart => self.vector(token),
            TokenKind::ByteVectorStart => self.bytevector(token),
            TokenKind::RParen => Err(SchemeError::syntax("unexpected ')'", Some(token.span))),
            TokenKind::Dot => Err(SchemeError::syntax("unexpected '.'", Some(token.span))),
            TokenKind::Eof => Err(SchemeError::syntax(
                "unexpected end of input",
                Some(token.span),
            )),
        }
    }

    fn list(&mut self, start: Token) -> Result<Datum, SchemeError> {
        if self.matches(&TokenKind::RParen)? {
            return Ok(Datum::EmptyList);
        }

        let mut items = Vec::new();
        let mut tail = Datum::EmptyList;

        loop {
            if self.check(&TokenKind::Eof)? {
                return Err(SchemeError::syntax("unterminated list", Some(start.span)));
            }

            if self.matches(&TokenKind::RParen)? {
                return Ok(Datum::list_with_tail(items, tail));
            }

            if self.matches(&TokenKind::Dot)? {
                tail = self.expression()?;
                self.expect(&TokenKind::RParen, "expected ')' after dotted pair")?;
                return Ok(Datum::list_with_tail(items, tail));
            }

            items.push(self.expression()?);
        }
    }

    fn vector(&mut self, start: Token) -> Result<Datum, SchemeError> {
        let mut items = Vec::new();

        loop {
            if self.check(&TokenKind::Eof)? {
                return Err(SchemeError::syntax("unterminated vector", Some(start.span)));
            }
            if self.matches(&TokenKind::RParen)? {
                return Ok(Datum::Vector(items));
            }
            if self.check(&TokenKind::Dot)? {
                return Err(SchemeError::syntax(
                    "vectors do not support dotted tail syntax",
                    Some(self.peek()?.span),
                ));
            }
            items.push(self.expression()?);
        }
    }

    fn bytevector(&mut self, start: Token) -> Result<Datum, SchemeError> {
        let mut bytes = Vec::new();

        loop {
            if self.check(&TokenKind::Eof)? {
                return Err(SchemeError::syntax(
                    "unterminated bytevector",
                    Some(start.span),
                ));
            }
            if self.matches(&TokenKind::RParen)? {
                return Ok(Datum::ByteVector(bytes));
            }

            let datum = self.expression()?;
            match datum {
                Datum::Number(value) if (0..=255).contains(&value) => bytes.push(value as u8),
                Datum::Number(_) => {
                    return Err(SchemeError::syntax(
                        "bytevector elements must be in range 0..=255",
                        Some(start.span),
                    ));
                }
                _ => {
                    return Err(SchemeError::syntax(
                        "bytevector elements must be numbers",
                        Some(start.span),
                    ));
                }
            }
        }
    }

    fn expect(&mut self, expected: &TokenKind, message: &str) -> Result<(), SchemeError> {
        if self.matches(expected)? {
            Ok(())
        } else {
            Err(SchemeError::syntax(message, Some(self.peek()?.span)))
        }
    }

    fn matches(&mut self, expected: &TokenKind) -> Result<bool, SchemeError> {
        if self.check(expected)? {
            self.advance()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn check(&mut self, expected: &TokenKind) -> Result<bool, SchemeError> {
        Ok(self.peek()?.kind.same_variant(expected))
    }

    fn advance(&mut self) -> Result<&Token, SchemeError> {
        if !self.is_at_end()? {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&mut self) -> Result<bool, SchemeError> {
        Ok(matches!(self.peek()?.kind, TokenKind::Eof))
    }

    fn peek(&mut self) -> Result<&Token, SchemeError> {
        self.ensure_loaded(self.current)?;
        Ok(&self.tokens[self.current])
    }

    fn previous(&self) -> Result<&Token, SchemeError> {
        self.tokens
            .get(self.current - 1)
            .ok_or_else(|| SchemeError::syntax("parser cursor has no previous token", None))
    }

    fn ensure_loaded(&mut self, index: usize) -> Result<(), SchemeError> {
        while self.tokens.len() <= index {
            let token = self.lexer.next_token()?;
            let is_eof = matches!(token.kind, TokenKind::Eof);
            self.tokens.push(token);
            if is_eof {
                break;
            }
        }
        Ok(())
    }
}

trait SameVariant {
    fn same_variant(&self, other: &Self) -> bool;
}

impl SameVariant for TokenKind {
    fn same_variant(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (TokenKind::LParen, TokenKind::LParen)
                | (TokenKind::RParen, TokenKind::RParen)
                | (TokenKind::Dot, TokenKind::Dot)
                | (TokenKind::Quote, TokenKind::Quote)
                | (TokenKind::Quasiquote, TokenKind::Quasiquote)
                | (TokenKind::Unquote, TokenKind::Unquote)
                | (TokenKind::UnquoteSplicing, TokenKind::UnquoteSplicing)
                | (TokenKind::VectorStart, TokenKind::VectorStart)
                | (TokenKind::ByteVectorStart, TokenKind::ByteVectorStart)
                | (TokenKind::Boolean(_), TokenKind::Boolean(_))
                | (TokenKind::Number(_), TokenKind::Number(_))
                | (TokenKind::Character(_), TokenKind::Character(_))
                | (TokenKind::String(_), TokenKind::String(_))
                | (TokenKind::Symbol(_), TokenKind::Symbol(_))
                | (TokenKind::Eof, TokenKind::Eof)
        )
    }
}
