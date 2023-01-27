use std::cmp::{min, max};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Span(logos::Span);

impl Span {
    pub(crate) fn new(s: logos::Span) -> Self {
        Self(s)
    }

    pub(crate) fn from_zero(end_byte: usize) -> Self {
        Self(0..end_byte)
    }

    pub fn byte_range(&self) -> &std::ops::Range<usize> {
        &self.0
    }

    pub(crate) fn merge(&self, other: &Self) -> Self {
        Self(min(self.0.start, other.0.start)..max(self.0.end, other.0.end))
    }

    pub(crate) fn end_empty_span(&self) -> Self {
        Self(self.0.end..self.0.end)
    }
}
