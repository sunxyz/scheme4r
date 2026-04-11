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
        let tokens = Lexer::new(self.source).tokenize()?;
        Parser::new(tokens).parse_all()
    }
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn parse_all(&mut self) -> Result<Vec<Datum>, SchemeError> {
        let mut forms = Vec::new();
        while !self.is_at_end() {
            forms.push(self.expression()?);
        }
        Ok(forms)
    }

    fn expression(&mut self) -> Result<Datum, SchemeError> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::Boolean(value) => Ok(Datum::Boolean(value)),
            TokenKind::Number(value) => Ok(Datum::Number(value)),
            TokenKind::String(value) => Ok(Datum::String(value)),
            TokenKind::Symbol(value) => Ok(Datum::Symbol(value)),
            TokenKind::Quote => {
                let quoted = self.expression()?;
                Ok(Datum::list(vec![Datum::symbol("quote"), quoted]))
            }
            TokenKind::LParen => self.list(token),
            TokenKind::RParen => Err(SchemeError::syntax("unexpected ')'", Some(token.span))),
            TokenKind::Dot => Err(SchemeError::syntax("unexpected '.'", Some(token.span))),
            TokenKind::Eof => Err(SchemeError::syntax(
                "unexpected end of input",
                Some(token.span),
            )),
        }
    }

    fn list(&mut self, start: Token) -> Result<Datum, SchemeError> {
        if self.matches(&TokenKind::RParen) {
            return Ok(Datum::EmptyList);
        }

        let mut items = Vec::new();
        let mut tail = Datum::EmptyList;

        loop {
            if self.check(&TokenKind::Eof) {
                return Err(SchemeError::syntax("unterminated list", Some(start.span)));
            }

            if self.matches(&TokenKind::RParen) {
                return Ok(Datum::list_with_tail(items, tail));
            }

            if self.matches(&TokenKind::Dot) {
                tail = self.expression()?;
                self.expect(&TokenKind::RParen, "expected ')' after dotted pair")?;
                return Ok(Datum::list_with_tail(items, tail));
            }

            items.push(self.expression()?);
        }
    }

    fn expect(&mut self, expected: &TokenKind, message: &str) -> Result<(), SchemeError> {
        if self.matches(expected) {
            Ok(())
        } else {
            Err(SchemeError::syntax(message, Some(self.peek().span)))
        }
    }

    fn matches(&mut self, expected: &TokenKind) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, expected: &TokenKind) -> bool {
        self.peek().kind.same_variant(expected)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
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
                | (TokenKind::Boolean(_), TokenKind::Boolean(_))
                | (TokenKind::Number(_), TokenKind::Number(_))
                | (TokenKind::String(_), TokenKind::String(_))
                | (TokenKind::Symbol(_), TokenKind::Symbol(_))
                | (TokenKind::Eof, TokenKind::Eof)
        )
    }
}
