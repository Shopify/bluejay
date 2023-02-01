use super::HasSpan;
use crate::Span;

#[derive(PartialEq, Debug)]
pub struct IntValue {
    value: i32,
    span: Span,
}

impl HasSpan for IntValue {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl Into<Span> for IntValue {
    fn into(self) -> Span {
        self.span
    }
}

impl Into<i32> for IntValue {
    fn into(self) -> i32 {
        self.value
    }
}

impl IntValue {
    pub(crate) fn value(&self) -> i32 {
        self.value
    }

    pub(crate) fn new(value: i32, span: Span) -> Self {
        Self { value, span }
    }
}

impl AsRef<i32> for IntValue {
    fn as_ref(&self) -> &i32 {
        &self.value
    }
}
