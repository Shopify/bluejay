use crate::lexical_token::HasSpan;
use crate::Span;

#[derive(PartialEq, Debug)]
pub struct FloatValue {
    value: f64,
    span: Span,
}

impl FloatValue {
    pub(crate) fn value(&self) -> f64 {
        self.value
    }

    pub(crate) fn new(value: f64, span: Span) -> Self {
        Self { value, span }
    }
}

impl HasSpan for FloatValue {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl From<FloatValue> for f64 {
    fn from(val: FloatValue) -> Self {
        val.value
    }
}

impl AsRef<f64> for FloatValue {
    fn as_ref(&self) -> &f64 {
        &self.value
    }
}
