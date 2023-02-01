use crate::ast::parse_error::ParseError;
use crate::lexical_token::{
    FloatValue, HasSpan, IntValue, LexicalToken, Name, PunctuatorType, StringValue,
};
use crate::scanner::ScanError;
use crate::{Scanner, Span};
use std::collections::VecDeque;

pub trait Tokens<'a>: Iterator<Item = LexicalToken<'a>> {
    fn expect_name(&mut self) -> Result<Name<'a>, ParseError>;
    fn expect_name_value(&mut self, value: &str) -> Result<Span, ParseError>;
    fn expect_punctuator(&mut self, punctuator_type: PunctuatorType) -> Result<Span, ParseError>;
    fn unexpected_eof(&self) -> ParseError;
    fn unexpected_token(&mut self) -> ParseError;
    fn next_if_punctuator(&mut self, punctuator_type: PunctuatorType) -> Option<Span>;
    fn next_if_int_value(&mut self) -> Option<IntValue>;
    fn next_if_float_value(&mut self) -> Option<FloatValue>;
    fn next_if_string_value(&mut self) -> Option<StringValue>;
    fn next_if_name(&mut self) -> Option<Name<'a>>;
    fn next_if_name_matches(&mut self, name: &str) -> Option<Span>;
    fn peek_name(&mut self, n: usize) -> Option<&Name>;
    fn peek_name_matches(&mut self, n: usize, name: &str) -> bool;
    fn peek_string_value(&mut self, n: usize) -> bool;
    fn peek_punctuator_matches(&mut self, n: usize, punctuator_type: PunctuatorType) -> bool;
}

pub struct ScannerTokens<'a, T: Scanner<'a>> {
    scanner: T,
    pub errors: Vec<ScanError>,
    buffer: VecDeque<LexicalToken<'a>>,
}

impl<'a, T: Scanner<'a>> ScannerTokens<'a, T> {
    pub fn new(scanner: T) -> Self {
        Self {
            scanner,
            errors: Vec::new(),
            buffer: VecDeque::new(),
        }
    }

    pub fn peek<'b, 'c: 'b>(&'c mut self, idx: usize) -> Option<&'b LexicalToken> {
        self.compute_up_to(idx);
        self.buffer.get(idx)
    }

    pub fn peek_next(&mut self) -> Option<&LexicalToken> {
        self.peek(0)
    }

    fn compute_up_to(&mut self, idx: usize) {
        while idx >= self.buffer.len() {
            match self.scanner.next() {
                Some(res) => match res {
                    Ok(val) => self.buffer.push_back(val),
                    Err(err) => self.errors.push(err),
                },
                None => break,
            }
        }
    }

    pub fn expect_name(&mut self) -> Result<Name<'a>, ParseError> {
        match self.next() {
            Some(LexicalToken::Name(n)) => Ok(n),
            Some(lt) => Err(ParseError::ExpectedName {
                span: lt.span().clone(),
            }),
            None => Err(self.unexpected_eof()),
        }
    }

    pub fn expect_name_value(&mut self, value: &str) -> Result<Span, ParseError> {
        match self.next() {
            Some(LexicalToken::Name(n)) if n.as_str() == value => Ok(n.span().clone()),
            Some(lt) => Err(ParseError::ExpectedIdentifier {
                span: lt.span().clone(),
                value: value.to_string(),
            }),
            None => Err(self.unexpected_eof()),
        }
    }

    pub fn expect_punctuator(
        &mut self,
        punctuator_type: PunctuatorType,
    ) -> Result<Span, ParseError> {
        match self.next() {
            Some(LexicalToken::Punctuator(p)) if p.r#type() == punctuator_type => Ok(p.into()),
            Some(lt) => Err(ParseError::ExpectedIdentifier {
                span: lt.into(),
                value: punctuator_type.to_string(),
            }),
            None => Err(self.unexpected_eof()),
        }
    }

    pub fn unexpected_eof(&self) -> ParseError {
        ParseError::UnexpectedEOF {
            span: self.scanner.empty_span(),
        }
    }

    pub fn unexpected_token(&mut self) -> ParseError {
        self.next()
            .map(|token| ParseError::UnexpectedToken { span: token.into() })
            .unwrap_or_else(|| self.unexpected_eof())
    }

    fn next_if<F>(&mut self, f: F) -> Option<Span>
    where
        F: Fn(&LexicalToken) -> bool,
    {
        match self.peek_next() {
            Some(token) if f(token) => {
                let span = token.span().clone();
                self.next();
                Some(span)
            }
            _ => None,
        }
    }

    pub fn next_if_punctuator(&mut self, punctuator_type: PunctuatorType) -> Option<Span> {
        self.next_if(|t| matches!(t, LexicalToken::Punctuator(p) if p.r#type() == punctuator_type))
    }

    pub fn next_if_int_value(&mut self) -> Option<IntValue> {
        matches!(self.peek_next(), Some(LexicalToken::IntValue(_)))
            .then(|| self.next().unwrap().into_int_value().unwrap())
    }

    pub fn next_if_float_value(&mut self) -> Option<FloatValue> {
        matches!(self.peek_next(), Some(LexicalToken::FloatValue(_)))
            .then(|| self.next().unwrap().into_float_value().unwrap())
    }

    pub fn next_if_string_value(&mut self) -> Option<StringValue> {
        matches!(self.peek_next(), Some(LexicalToken::StringValue(_)))
            .then(|| self.next().unwrap().into_string_value().unwrap())
    }

    pub fn next_if_name(&mut self) -> Option<Name<'a>> {
        matches!(self.peek_next(), Some(LexicalToken::Name(_)))
            .then(|| self.next().unwrap().into_name().unwrap())
    }

    pub fn next_if_name_matches(&mut self, name: &str) -> Option<Span> {
        self.next_if(|t| matches!(t, LexicalToken::Name(n) if n.as_str() == name))
    }

    pub fn peek_name(&mut self, n: usize) -> Option<&Name> {
        self.peek(n).and_then(LexicalToken::as_name)
    }

    pub fn peek_name_matches(&mut self, n: usize, name: &str) -> bool {
        matches!(self.peek_name(n), Some(n) if n.as_str() == name)
    }

    pub fn peek_string_value(&mut self, n: usize) -> bool {
        matches!(self.peek(n), Some(LexicalToken::StringValue(_)))
    }

    pub fn peek_punctuator_matches(&mut self, n: usize, punctuator_type: PunctuatorType) -> bool {
        matches!(self.peek(n), Some(LexicalToken::Punctuator(p)) if p.r#type() == punctuator_type)
    }
}

impl<'a, T: Scanner<'a>> Iterator for ScannerTokens<'a, T> {
    type Item = LexicalToken<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.compute_up_to(0);
        self.buffer.pop_front()
    }
}

impl<'a, T: Scanner<'a>> From<ScannerTokens<'a, T>> for Vec<ScanError> {
    fn from(val: ScannerTokens<'a, T>) -> Self {
        val.errors
    }
}

impl<'a, T: Scanner<'a>> Tokens<'a> for ScannerTokens<'a, T> {
    fn expect_name(&mut self) -> Result<Name<'a>, ParseError> {
        self.expect_name()
    }

    fn expect_name_value(&mut self, value: &str) -> Result<Span, ParseError> {
        self.expect_name_value(value)
    }

    fn expect_punctuator(&mut self, punctuator_type: PunctuatorType) -> Result<Span, ParseError> {
        self.expect_punctuator(punctuator_type)
    }

    fn unexpected_eof(&self) -> ParseError {
        self.unexpected_eof()
    }

    fn unexpected_token(&mut self) -> ParseError {
        self.unexpected_token()
    }

    fn next_if_punctuator(&mut self, punctuator_type: PunctuatorType) -> Option<Span> {
        self.next_if_punctuator(punctuator_type)
    }

    fn next_if_int_value(&mut self) -> Option<IntValue> {
        self.next_if_int_value()
    }

    fn next_if_float_value(&mut self) -> Option<FloatValue> {
        self.next_if_float_value()
    }

    fn next_if_string_value(&mut self) -> Option<StringValue> {
        self.next_if_string_value()
    }

    fn next_if_name(&mut self) -> Option<Name<'a>> {
        self.next_if_name()
    }

    fn next_if_name_matches(&mut self, name: &str) -> Option<Span> {
        self.next_if_name_matches(name)
    }

    fn peek_name(&mut self, n: usize) -> Option<&Name> {
        self.peek_name(n)
    }

    fn peek_name_matches(&mut self, n: usize, name: &str) -> bool {
        self.peek_name_matches(n, name)
    }

    fn peek_string_value(&mut self, n: usize) -> bool {
        self.peek_string_value(n)
    }

    fn peek_punctuator_matches(&mut self, n: usize, punctuator_type: PunctuatorType) -> bool {
        self.peek_punctuator_matches(n, punctuator_type)
    }
}
