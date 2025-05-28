use crate::ast::parse_error::ParseError;
use crate::lexer::{LexError, Lexer};
use crate::lexical_token::{
    FloatValue, IntValue, LexicalToken, Name, PunctuatorType, StringValue, Variable,
};
use crate::{HasSpan, Span};
use std::collections::VecDeque;

pub trait Tokens<'a>: Iterator<Item = LexicalToken<'a>> {
    fn expect_variable(&mut self) -> Result<Variable<'a>, ParseError>;
    fn expect_name(&mut self) -> Result<Name<'a>, ParseError>;
    fn expect_name_value(&mut self, value: &str) -> Result<Span, ParseError>;
    fn expect_punctuator(&mut self, punctuator_type: PunctuatorType) -> Result<Span, ParseError>;
    fn unexpected_eof(&self) -> ParseError;
    fn unexpected_token(&mut self) -> ParseError;
    fn next_if_punctuator(&mut self, punctuator_type: PunctuatorType) -> Option<Span>;
    fn next_if_int_value(&mut self) -> Option<IntValue>;
    fn next_if_float_value(&mut self) -> Option<FloatValue>;
    fn next_if_string_value(&mut self) -> Option<StringValue<'a>>;
    fn next_if_name(&mut self) -> Option<Name<'a>>;
    fn next_if_name_matches(&mut self, name: &str) -> Option<Span>;
    fn peek_variable_name(&mut self, n: usize) -> bool;
    fn peek_name(&mut self, n: usize) -> Option<&Name>;
    fn peek_name_matches(&mut self, n: usize, name: &str) -> bool;
    fn peek_string_value(&mut self, n: usize) -> bool;
    fn peek_punctuator_matches(&mut self, n: usize, punctuator_type: PunctuatorType) -> bool;
    fn into_errors(self) -> Vec<(LexError, Span)>;
    fn token_count(&self) -> usize;
}

pub struct LexerTokens<'a, T: Lexer<'a>> {
    lexer: T,
    errors: Vec<(LexError, Span)>,
    buffer: VecDeque<LexicalToken<'a>>,
}

impl<'a, T: Lexer<'a>> LexerTokens<'a, T> {
    #[inline]
    pub fn new(lexer: T) -> Self {
        Self {
            lexer,
            errors: Vec::new(),
            buffer: VecDeque::new(),
        }
    }

    #[inline]
    pub fn token_count(&self) -> usize {
        self.lexer.token_count()
    }

    #[inline]
    pub fn peek<'b, 'c: 'b>(&'c mut self, idx: usize) -> Option<&'b LexicalToken<'c>> {
        self.compute_up_to(idx);
        self.buffer.get(idx)
    }

    #[inline]
    pub fn peek_next(&mut self) -> Option<&LexicalToken> {
        self.peek(0)
    }

    fn compute_up_to(&mut self, idx: usize) {
        while idx >= self.buffer.len() {
            match self.lexer.next() {
                Some(res) => match res {
                    Ok(val) => self.buffer.push_back(val),
                    Err(err) => self.errors.push(err),
                },
                None => break,
            }
        }
    }

    #[inline]
    pub fn expect_name(&mut self) -> Result<Name<'a>, ParseError> {
        match self.next() {
            Some(LexicalToken::Name(n)) => Ok(n),
            Some(lt) => Err(ParseError::ExpectedName { span: lt.into() }),
            None => Err(self.unexpected_eof()),
        }
    }

    #[inline]
    pub fn expect_variable(&mut self) -> Result<Variable<'a>, ParseError> {
        match self.next() {
            Some(LexicalToken::VariableName(n)) => Ok(n),
            Some(lt) => Err(ParseError::ExpectedIdentifier {
                span: lt.into(),
                value: String::from("$"),
            }),
            None => Err(self.unexpected_eof()),
        }
    }

    #[inline]
    pub fn expect_name_value(&mut self, value: &str) -> Result<Span, ParseError> {
        match self.next() {
            Some(LexicalToken::Name(n)) if n.as_str() == value => Ok(n.into()),
            Some(lt) => Err(ParseError::ExpectedIdentifier {
                span: lt.into(),
                value: value.to_string(),
            }),
            None => Err(self.unexpected_eof()),
        }
    }

    #[inline]
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

    #[inline]
    pub fn unexpected_eof(&self) -> ParseError {
        ParseError::UnexpectedEOF {
            span: self.lexer.empty_span(),
        }
    }

    #[inline]
    pub fn unexpected_token(&mut self) -> ParseError {
        self.next()
            .map(|token| ParseError::UnexpectedToken { span: token.into() })
            .unwrap_or_else(|| self.unexpected_eof())
    }

    #[inline]
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

    #[inline]
    pub fn next_if_punctuator(&mut self, punctuator_type: PunctuatorType) -> Option<Span> {
        self.next_if(|t| matches!(t, LexicalToken::Punctuator(p) if p.r#type() == punctuator_type))
    }

    #[inline]
    pub fn next_if_int_value(&mut self) -> Option<IntValue> {
        matches!(self.peek_next(), Some(LexicalToken::IntValue(_)))
            .then(|| self.next().unwrap().into_int_value().unwrap())
    }

    #[inline]
    pub fn next_if_float_value(&mut self) -> Option<FloatValue> {
        matches!(self.peek_next(), Some(LexicalToken::FloatValue(_)))
            .then(|| self.next().unwrap().into_float_value().unwrap())
    }

    #[inline]
    pub fn next_if_string_value(&mut self) -> Option<StringValue<'a>> {
        matches!(self.peek_next(), Some(LexicalToken::StringValue(_)))
            .then(|| self.next().unwrap().into_string_value().unwrap())
    }

    #[inline]
    pub fn next_if_name(&mut self) -> Option<Name<'a>> {
        matches!(self.peek_next(), Some(LexicalToken::Name(_)))
            .then(|| self.next().unwrap().into_name().unwrap())
    }

    #[inline]
    pub fn next_if_name_matches(&mut self, name: &str) -> Option<Span> {
        self.next_if(|t| matches!(t, LexicalToken::Name(n) if n.as_str() == name))
    }

    #[inline]
    pub fn peek_name(&mut self, n: usize) -> Option<&Name> {
        self.peek(n).and_then(LexicalToken::as_name)
    }

    #[inline]
    pub fn peek_variable_name(&mut self, n: usize) -> bool {
        matches!(self.peek(n), Some(LexicalToken::VariableName(_)))
    }

    #[inline]
    pub fn peek_name_matches(&mut self, n: usize, name: &str) -> bool {
        matches!(self.peek_name(n), Some(n) if n.as_str() == name)
    }

    #[inline]
    pub fn peek_string_value(&mut self, n: usize) -> bool {
        matches!(self.peek(n), Some(LexicalToken::StringValue(_)))
    }

    #[inline]
    pub fn peek_punctuator_matches(&mut self, n: usize, punctuator_type: PunctuatorType) -> bool {
        matches!(self.peek(n), Some(LexicalToken::Punctuator(p)) if p.r#type() == punctuator_type)
    }
}

impl<'a, T: Lexer<'a>> Iterator for LexerTokens<'a, T> {
    type Item = LexicalToken<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.compute_up_to(0);
        self.buffer.pop_front()
    }
}

impl<'a, T: Lexer<'a>> From<LexerTokens<'a, T>> for Vec<(LexError, Span)> {
    fn from(val: LexerTokens<'a, T>) -> Self {
        val.errors
    }
}

impl<'a, T: Lexer<'a>> Tokens<'a> for LexerTokens<'a, T> {
    #[inline]
    fn expect_variable(&mut self) -> Result<Variable<'a>, ParseError> {
        self.expect_variable()
    }

    #[inline]
    fn expect_name(&mut self) -> Result<Name<'a>, ParseError> {
        self.expect_name()
    }

    #[inline]
    fn expect_name_value(&mut self, value: &str) -> Result<Span, ParseError> {
        self.expect_name_value(value)
    }

    #[inline]
    fn expect_punctuator(&mut self, punctuator_type: PunctuatorType) -> Result<Span, ParseError> {
        self.expect_punctuator(punctuator_type)
    }

    #[inline]
    fn unexpected_eof(&self) -> ParseError {
        self.unexpected_eof()
    }

    #[inline]
    fn unexpected_token(&mut self) -> ParseError {
        self.unexpected_token()
    }

    #[inline]
    fn next_if_punctuator(&mut self, punctuator_type: PunctuatorType) -> Option<Span> {
        self.next_if_punctuator(punctuator_type)
    }

    #[inline]
    fn next_if_int_value(&mut self) -> Option<IntValue> {
        self.next_if_int_value()
    }

    #[inline]
    fn next_if_float_value(&mut self) -> Option<FloatValue> {
        self.next_if_float_value()
    }

    #[inline]
    fn next_if_string_value(&mut self) -> Option<StringValue<'a>> {
        self.next_if_string_value()
    }

    #[inline]
    fn next_if_name(&mut self) -> Option<Name<'a>> {
        self.next_if_name()
    }

    #[inline]
    fn next_if_name_matches(&mut self, name: &str) -> Option<Span> {
        self.next_if_name_matches(name)
    }

    #[inline]
    fn peek_name(&mut self, n: usize) -> Option<&Name> {
        self.peek_name(n)
    }

    #[inline]
    fn peek_variable_name(&mut self, n: usize) -> bool {
        self.peek_variable_name(n)
    }

    #[inline]
    fn peek_name_matches(&mut self, n: usize, name: &str) -> bool {
        self.peek_name_matches(n, name)
    }

    #[inline]
    fn peek_string_value(&mut self, n: usize) -> bool {
        self.peek_string_value(n)
    }

    #[inline]
    fn peek_punctuator_matches(&mut self, n: usize, punctuator_type: PunctuatorType) -> bool {
        self.peek_punctuator_matches(n, punctuator_type)
    }

    #[inline]
    fn into_errors(self) -> Vec<(LexError, Span)> {
        self.errors
    }

    #[inline]
    fn token_count(&self) -> usize {
        self.token_count()
    }
}
