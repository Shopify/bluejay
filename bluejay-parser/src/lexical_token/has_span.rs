use crate::Span;

pub trait HasSpan {
    fn span(&self) -> &Span;
}
