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
            self.skip_whitespace_and_comments();

            let span = Span::new(self.line, self.column);
            let Some(ch) = self.peek() else {
                tokens.push(Token::new(TokenKind::Eof, span));
                break;
            };

            match ch {
                '(' => {
                    self.advance();
                    tokens.push(Token::new(TokenKind::LParen, span));
                }
                ')' => {
                    self.advance();
                    tokens.push(Token::new(TokenKind::RParen, span));
                }
                '\'' => {
                    self.advance();
                    tokens.push(Token::new(TokenKind::Quote, span));
                }
                '"' => tokens.push(self.read_string()?),
                '#' => tokens.push(self.read_dispatch()?),
                '.' if match self.peek_next() {
                    Some(next) => is_delimiter(next),
                    None => true,
                } =>
                {
                    self.advance();
                    tokens.push(Token::new(TokenKind::Dot, span));
                }
                _ => tokens.push(self.read_atom()),
            }
        }

        Ok(tokens)
    }

    fn read_string(&mut self) -> Result<Token, SchemeError> {
        let span = Span::new(self.line, self.column);
        self.advance();
        let mut value = String::new();

        while let Some(ch) = self.peek() {
            match ch {
                '"' => {
                    self.advance();
                    return Ok(Token::new(TokenKind::String(value), span));
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

    fn read_dispatch(&mut self) -> Result<Token, SchemeError> {
        let span = Span::new(self.line, self.column);
        self.advance();

        match self.peek() {
            Some('t') => {
                self.advance();
                Ok(Token::new(TokenKind::Boolean(true), span))
            }
            Some('f') => {
                self.advance();
                Ok(Token::new(TokenKind::Boolean(false), span))
            }
            _ => Err(SchemeError::read(
                "unsupported dispatch syntax in minimal kernel",
                Some(span),
            )),
        }
    }

    fn read_atom(&mut self) -> Token {
        let span = Span::new(self.line, self.column);
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

        Token::new(kind, span)
    }

    fn skip_whitespace_and_comments(&mut self) {
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
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.index + 1).copied()
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
    ch.is_whitespace() || matches!(ch, '(' | ')' | '"' | '\'' | ';')
}
