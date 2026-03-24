use std::cmp::{max, min};
use std::cmp::{Ord, Ordering, PartialOrd};
use std::ops::{Add, Range};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    start: u32,
    len: u32,
}

impl Span {
    #[inline]
    pub(crate) fn new(s: logos::Span) -> Self {
        Self {
            start: s.start as u32,
            len: (s.end - s.start) as u32,
        }
    }

    #[inline]
    pub fn byte_range(&self) -> Range<usize> {
        self.start as usize..(self.start + self.len) as usize
    }

    #[inline]
    pub fn merge(&self, other: &Self) -> Self {
        let start = min(self.start, other.start);
        let end = max(self.start + self.len, other.start + other.len);
        Self {
            start,
            len: end - start,
        }
    }
}

impl From<Span> for Range<usize> {
    fn from(val: Span) -> Self {
        val.byte_range()
    }
}

impl From<logos::Span> for Span {
    #[inline]
    fn from(value: logos::Span) -> Self {
        Self::new(value)
    }
}

impl Add<usize> for Span {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self {
            start: self.start + rhs as u32,
            len: self.len,
        }
    }
}

impl Ord for Span {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start)
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
