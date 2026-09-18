use super::HasSpan;
use crate::Span;
use bluejay_core::IntegerValue;

#[derive(PartialEq, Debug)]
pub struct IntValue<'a> {
    value: &'a str,
    span: Span,
}

impl HasSpan for IntValue<'_> {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl From<IntValue<'_>> for Span {
    fn from(value: IntValue<'_>) -> Self {
        value.span
    }
}

impl<'a> From<IntValue<'a>> for IntegerValue<'a> {
    fn from(val: IntValue<'a>) -> Self {
        Self::from_validated_literal(val.value)
    }
}

impl<'a> IntValue<'a> {
    pub(crate) fn new(value: &'a str, span: Span) -> Self {
        Self { value, span }
    }
}

impl AsRef<str> for IntValue<'_> {
    fn as_ref(&self) -> &str {
        self.value
    }
}
