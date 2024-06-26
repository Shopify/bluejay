use crate::lexical_token::HasSpan;
use crate::Span;

#[derive(PartialEq, Debug)]
pub struct BooleanValue {
    value: bool,
    span: Span,
}

impl BooleanValue {
    pub(crate) fn value(&self) -> bool {
        self.value
    }

    pub(crate) fn new(value: bool, span: Span) -> Self {
        Self { value, span }
    }
}

impl HasSpan for BooleanValue {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl From<BooleanValue> for Span {
    fn from(value: BooleanValue) -> Self {
        value.span
    }
}

impl From<BooleanValue> for bool {
    fn from(val: BooleanValue) -> Self {
        val.value
    }
}

impl AsRef<bool> for BooleanValue {
    fn as_ref(&self) -> &bool {
        &self.value
    }
}
