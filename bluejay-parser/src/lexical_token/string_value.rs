use super::HasSpan;
use crate::Span;
use std::borrow::Cow;

#[derive(PartialEq, Debug)]
pub struct StringValue<'a> {
    contents: Cow<'a, str>,
    span: Span,
}

impl<'a> StringValue<'a> {
    pub fn as_str(&self) -> &str {
        self.contents.as_ref()
    }

    pub(crate) fn new(contents: Cow<'a, str>, span: Span) -> Self {
        Self { contents, span }
    }
}

impl<'a> ToString for StringValue<'a> {
    fn to_string(&self) -> String {
        self.contents.to_string()
    }
}

impl<'a> HasSpan for StringValue<'a> {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl<'a> From<StringValue<'a>> for Span {
    fn from(value: StringValue<'a>) -> Self {
        value.span
    }
}

impl<'a> AsRef<str> for StringValue<'a> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
