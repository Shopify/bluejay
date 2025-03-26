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

impl HasSpan for StringValue<'_> {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl<'a> From<StringValue<'a>> for Span {
    fn from(value: StringValue<'a>) -> Self {
        value.span
    }
}

impl AsRef<str> for StringValue<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
