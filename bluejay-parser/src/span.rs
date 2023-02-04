use std::cmp::{max, min};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Span(logos::Span);

impl Span {
    pub(crate) fn new(s: logos::Span) -> Self {
        Self(s)
    }

    pub fn byte_range(&self) -> &std::ops::Range<usize> {
        &self.0
    }

    pub(crate) fn merge(&self, other: &Self) -> Self {
        Self(min(self.0.start, other.0.start)..max(self.0.end, other.0.end))
    }

    pub(crate) fn empty() -> Self {
        Self(0..0)
    }
}
