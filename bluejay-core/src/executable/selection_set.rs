use crate::executable::AbstractSelection;
use std::cmp::{Eq, Ord};
use std::hash::Hash;

pub trait SelectionSet: AsRef<[Self::Selection]> + Hash + Eq + Ord {
    type Selection: AbstractSelection;
}
