use super::HasSpan;
use crate::Span;
use std::cmp::PartialEq;

#[derive(PartialEq, Debug, Clone)]
pub struct Name<'a> {
    value: &'a str,
    span: Span,
}

impl<'a> Name<'a> {
    pub fn as_str(&self) -> &'a str {
        self.value
    }

    pub(crate) fn new(value: &'a str, span: Span) -> Self {
        Self { value, span }
    }
}

impl<'a> HasSpan for Name<'a> {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl<'a> From<Name<'a>> for Span {
    fn from(value: Name<'a>) -> Self {
        value.span
    }
}

impl<'a> AsRef<str> for Name<'a> {
    fn as_ref(&self) -> &str {
        self.value
    }
}

impl<'a> PartialEq<str> for Name<'a> {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}
