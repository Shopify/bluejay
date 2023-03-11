use super::HasSpan;
use crate::Span;

#[derive(PartialEq, Debug)]
pub struct StringValue {
    contents: String,
    span: Span,
}

impl StringValue {
    pub fn as_str(&self) -> &str {
        &self.contents
    }

    pub(crate) fn new(contents: String, span: Span) -> Self {
        Self { contents, span }
    }
}

impl ToString for StringValue {
    fn to_string(&self) -> String {
        self.contents.to_string()
    }
}

impl HasSpan for StringValue {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl AsRef<str> for StringValue {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
