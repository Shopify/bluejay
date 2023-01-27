use crate::Span;
use crate::lexical_token::HasSpan;

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
    fn span(&self) -> &Span {
        &self.span
    }
}

impl Into<Span> for FloatValue {
    fn into(self) -> Span {
        self.span
    }
}

impl Into<f64> for FloatValue {
    fn into(self) -> f64 {
        self.value
    }
}

impl AsRef<f64> for FloatValue {
    fn as_ref(&self) -> &f64 {
        &self.value
    }
}
