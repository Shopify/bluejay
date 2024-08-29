use std::cmp::{max, min};
use std::cmp::{Ord, Ordering, PartialOrd};
use std::ops::{Add, Range};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Span(logos::Span);

impl Span {
    #[inline]
    pub(crate) fn new(s: logos::Span) -> Self {
        Self(s)
    }

    #[inline]
    pub fn byte_range(&self) -> &Range<usize> {
        &self.0
    }

    #[inline]
    pub fn merge(&self, other: &Self) -> Self {
        Self(min(self.0.start, other.0.start)..max(self.0.end, other.0.end))
    }
}

impl From<Span> for Range<usize> {
    fn from(val: Span) -> Self {
        val.0
    }
}

impl From<logos::Span> for Span {
    #[inline]
    fn from(value: logos::Span) -> Self {
        Self(value)
    }
}

impl Add<usize> for Span {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self((self.0.start + rhs)..(self.0.end + rhs))
    }
}

impl Ord for Span {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.start.cmp(&other.0.start)
    }
}

impl PartialOrd for Span {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub trait HasSpan {
    fn span(&self) -> &Span;
}
