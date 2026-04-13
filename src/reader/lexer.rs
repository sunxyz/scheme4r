use crate::{
    error::SchemeError,
    reader::{
        span::Span,
        token::{Token, TokenKind},
    },
};

pub struct Lexer<'a> {
    chars: Vec<char>,
    index: usize,
    line: usize,
    column: usize,
    _source: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().collect(),
            index: 0,
            line: 1,
            column: 1,
            _source: source,
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, SchemeError> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token.kind, TokenKind::Eof);
            tokens.push(token);
            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }

    pub(crate) fn next_token(&mut self) -> Result<Token, SchemeError> {
        self.skip_whitespace_and_comments()?;

        let span = Span::new(self.line, self.column);
        let start = self.index;
        let Some(ch) = self.peek() else {
            return Ok(Token::new(TokenKind::Eof, span, start, start));
        };

        match ch {
            '(' => {
                self.advance();
                Ok(Token::new(TokenKind::LParen, span, start, self.index))
            }
            ')' => {
                self.advance();
                Ok(Token::new(TokenKind::RParen, span, start, self.index))
            }
            '\'' => {
                self.advance();
                Ok(Token::new(TokenKind::Quote, span, start, self.index))
            }
            '`' => {
                self.advance();
                Ok(Token::new(TokenKind::Quasiquote, span, start, self.index))
            }
            ',' => {
                self.advance();
                if self.peek() == Some('@') {
                    self.advance();
                    Ok(Token::new(
                        TokenKind::UnquoteSplicing,
                        span,
                        start,
                        self.index,
                    ))
                } else {
                    Ok(Token::new(TokenKind::Unquote, span, start, self.index))
                }
            }
            '"' => self.read_string(span, start),
            '#' => self.read_dispatch(span, start),
            '.' if match self.peek_next() {
                Some(next) => is_delimiter(next),
                None => true,
            } =>
            {
                self.advance();
                Ok(Token::new(TokenKind::Dot, span, start, self.index))
            }
            _ => Ok(self.read_atom(span, start)),
        }
    }

    fn read_string(&mut self, span: Span, start: usize) -> Result<Token, SchemeError> {
        self.advance();
        let mut value = String::new();

        while let Some(ch) = self.peek() {
            match ch {
                '"' => {
                    self.advance();
                    return Ok(Token::new(
                        TokenKind::String(value),
                        span,
                        start,
                        self.index,
                    ));
                }
                '\\' => {
                    self.advance();
                    let escaped = self.peek().ok_or_else(|| {
                        SchemeError::read("unterminated string literal", Some(span))
                    })?;
                    let escaped = match escaped {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '"' => '"',
                        '\\' => '\\',
                        other => other,
                    };
                    self.advance();
                    value.push(escaped);
                }
                _ => {
                    value.push(ch);
                    self.advance();
                }
            }
        }

        Err(SchemeError::read("unterminated string literal", Some(span)))
    }

    fn read_dispatch(&mut self, span: Span, start: usize) -> Result<Token, SchemeError> {
        self.advance();

        match self.peek() {
            Some('t') => {
                self.advance();
                Ok(Token::new(
                    TokenKind::Boolean(true),
                    span,
                    start,
                    self.index,
                ))
            }
            Some('f') => {
                self.advance();
                Ok(Token::new(
                    TokenKind::Boolean(false),
                    span,
                    start,
                    self.index,
                ))
            }
            Some('(') => {
                self.advance();
                Ok(Token::new(TokenKind::VectorStart, span, start, self.index))
            }
            Some('u') if self.peek_n(1) == Some('8') && self.peek_n(2) == Some('(') => {
                self.advance();
                self.advance();
                self.advance();
                Ok(Token::new(
                    TokenKind::ByteVectorStart,
                    span,
                    start,
                    self.index,
                ))
            }
            Some('\\') => self.read_character(span, start),
            Some(ch) if matches!(ch, 'b' | 'o' | 'd' | 'x' | 'e' | 'i') => {
                self.read_prefixed_number(span, start)
            }
            _ => Err(SchemeError::read("unsupported dispatch syntax", Some(span))),
        }
    }

    fn read_prefixed_number(&mut self, span: Span, start: usize) -> Result<Token, SchemeError> {
        let mut radix = 10_u32;
        let mut saw_radix = false;
        let mut saw_exactness = false;

        loop {
            match self.peek() {
                Some('#') => {
                    self.advance();
                }
                Some('b') if !saw_radix => {
                    self.advance();
                    radix = 2;
                    saw_radix = true;
                }
                Some('o') if !saw_radix => {
                    self.advance();
                    radix = 8;
                    saw_radix = true;
                }
                Some('d') if !saw_radix => {
                    self.advance();
                    radix = 10;
                    saw_radix = true;
                }
                Some('x') if !saw_radix => {
                    self.advance();
                    radix = 16;
                    saw_radix = true;
                }
                Some('e' | 'i') if !saw_exactness => {
                    self.advance();
                    saw_exactness = true;
                }
                _ => break,
            }
        }

        let mut negative = false;
        match self.peek() {
            Some('+') => {
                self.advance();
            }
            Some('-') => {
                self.advance();
                negative = true;
            }
            _ => {}
        }

        let mut digits = String::new();
        while let Some(ch) = self.peek() {
            if is_delimiter(ch) {
                break;
            }
            digits.push(ch);
            self.advance();
        }

        if digits.is_empty() {
            return Err(SchemeError::read(
                "numeric prefix must be followed by digits",
                Some(span),
            ));
        }

        let parsed = i64::from_str_radix(&digits, radix)
            .map_err(|_| SchemeError::read("invalid prefixed number literal", Some(span)))?;
        let value = if negative { -parsed } else { parsed };
        Ok(Token::new(
            TokenKind::Number(value),
            span,
            start,
            self.index,
        ))
    }

    fn read_character(&mut self, span: Span, start: usize) -> Result<Token, SchemeError> {
        self.advance();
        let mut text = String::new();

        while let Some(ch) = self.peek() {
            if is_delimiter(ch) {
                break;
            }
            text.push(ch);
            self.advance();
        }

        let value = match text.as_str() {
            "space" => ' ',
            "newline" => '\n',
            "" => {
                return Err(SchemeError::read(
                    "character literal requires a value",
                    Some(span),
                ));
            }
            _ => {
                let mut chars = text.chars();
                let ch = chars.next().ok_or_else(|| {
                    SchemeError::read("character literal requires a value", Some(span))
                })?;
                if chars.next().is_some() {
                    return Err(SchemeError::read(
                        "unsupported named character literal",
                        Some(span),
                    ));
                }
                ch
            }
        };

        Ok(Token::new(
            TokenKind::Character(value),
            span,
            start,
            self.index,
        ))
    }

    fn read_atom(&mut self, span: Span, start: usize) -> Token {
        let mut text = String::new();

        while let Some(ch) = self.peek() {
            if is_delimiter(ch) {
                break;
            }
            text.push(ch);
            self.advance();
        }

        let kind = match text.parse::<i64>() {
            Ok(number) => TokenKind::Number(number),
            Err(_) => TokenKind::Symbol(text),
        };

        Token::new(kind, span, start, self.index)
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), SchemeError> {
        loop {
            match self.peek() {
                Some(ch) if ch.is_whitespace() => {
                    self.advance();
                }
                Some(';') => {
                    while let Some(ch) = self.peek() {
                        self.advance();
                        if ch == '\n' {
                            break;
                        }
                    }
                }
                Some('#') if self.peek_next() == Some('|') => {
                    self.advance();
                    self.advance();
                    self.skip_block_comment()?;
                }
                Some('#') if self.peek_next() == Some(';') => {
                    self.advance();
                    self.advance();
                    self.skip_datum()?;
                }
                _ => break,
            }
        }
        Ok(())
    }

    fn skip_block_comment(&mut self) -> Result<(), SchemeError> {
        let span = Span::new(self.line, self.column);
        let mut depth = 1_usize;

        while let Some(ch) = self.peek() {
            match (ch, self.peek_next()) {
                ('#', Some('|')) => {
                    self.advance();
                    self.advance();
                    depth += 1;
                }
                ('|', Some('#')) => {
                    self.advance();
                    self.advance();
                    depth -= 1;
                    if depth == 0 {
                        return Ok(());
                    }
                }
                _ => {
                    self.advance();
                }
            }
        }

        Err(SchemeError::read("unterminated block comment", Some(span)))
    }

    fn skip_datum(&mut self) -> Result<(), SchemeError> {
        self.skip_whitespace_and_comments()?;
        let span = Span::new(self.line, self.column);
        let Some(ch) = self.peek() else {
            return Err(SchemeError::read(
                "datum comment requires a following datum",
                Some(span),
            ));
        };

        match ch {
            '(' => {
                self.advance();
                self.skip_list_like_datum()
            }
            '\'' | '`' => {
                self.advance();
                self.skip_datum()
            }
            ',' => {
                self.advance();
                if self.peek() == Some('@') {
                    self.advance();
                }
                self.skip_datum()
            }
            '"' => self.skip_string_literal(span),
            '#' if self.peek_next() == Some('(') => {
                self.advance();
                self.advance();
                self.skip_list_like_datum()
            }
            '#' if self.peek_next() == Some('u')
                && self.peek_n(2) == Some('8')
                && self.peek_n(3) == Some('(') =>
            {
                self.advance();
                self.advance();
                self.advance();
                self.advance();
                self.skip_list_like_datum()
            }
            '#' if self.peek_next() == Some('\\') => {
                self.advance();
                self.advance();
                self.skip_atom_like();
                Ok(())
            }
            '#' if self.peek_next() == Some('t') || self.peek_next() == Some('f') => {
                self.advance();
                self.advance();
                Ok(())
            }
            _ => {
                self.skip_atom_like();
                Ok(())
            }
        }
    }

    fn skip_list_like_datum(&mut self) -> Result<(), SchemeError> {
        let span = Span::new(self.line, self.column);

        loop {
            self.skip_whitespace_and_comments()?;
            match self.peek() {
                Some(')') => {
                    self.advance();
                    return Ok(());
                }
                Some(_) => self.skip_datum()?,
                None => {
                    return Err(SchemeError::read(
                        "unterminated commented list datum",
                        Some(span),
                    ));
                }
            }
        }
    }

    fn skip_string_literal(&mut self, span: Span) -> Result<(), SchemeError> {
        self.advance();
        while let Some(ch) = self.peek() {
            match ch {
                '"' => {
                    self.advance();
                    return Ok(());
                }
                '\\' => {
                    self.advance();
                    if self.peek().is_none() {
                        return Err(SchemeError::read("unterminated string literal", Some(span)));
                    }
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }

        Err(SchemeError::read("unterminated string literal", Some(span)))
    }

    fn skip_atom_like(&mut self) {
        while let Some(ch) = self.peek() {
            if is_delimiter(ch) {
                break;
            }
            self.advance();
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.index + 1).copied()
    }

    fn peek_n(&self, offset: usize) -> Option<char> {
        self.chars.get(self.index + offset).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.index += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }
}

fn is_delimiter(ch: char) -> bool {
    ch.is_whitespace() || matches!(ch, '(' | ')' | '"' | '\'' | '`' | ',' | ';')
}
