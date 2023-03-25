use crate::Span;
use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub struct Annotation {
    message: Cow<'static, str>,
    span: Span,
}

impl Annotation {
    pub fn new(message: impl Into<Cow<'static, str>>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }

    pub fn message(&self) -> &str {
        self.message.as_ref()
    }

    pub fn span(&self) -> &Span {
        &self.span
    }
}
