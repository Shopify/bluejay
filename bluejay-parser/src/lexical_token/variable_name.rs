use super::HasSpan;
use crate::Span;
use std::cmp::PartialEq;

#[derive(PartialEq, Debug, Clone)]
pub struct Variable<'a> {
    /// A value representing the name of the variable
    /// stripped of the dollar sign.
    value: &'a str,
    /// A span representing the position of the variable
    /// in the source string, including the dollar sign.
    span: Span,
}

impl<'a> Variable<'a> {
    pub fn name(&self) -> &'a str {
        self.value
    }

    pub fn as_str(&self) -> &'a str {
        self.value
    }

    pub(crate) fn new(value: &'a str, span: Span) -> Self {
        Self { value, span }
    }
}

impl<'a> HasSpan for Variable<'a> {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl<'a> From<Variable<'a>> for Span {
    fn from(value: Variable<'a>) -> Self {
        value.span
    }
}

impl<'a> AsRef<str> for Variable<'a> {
    fn as_ref(&self) -> &str {
        self.value
    }
}

impl<'a> PartialEq<str> for Variable<'a> {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}
