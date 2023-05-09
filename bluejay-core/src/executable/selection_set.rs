use crate::executable::AbstractSelection;
use crate::AsIter;
use std::cmp::{Eq, Ord};
use std::hash::Hash;

pub trait SelectionSet: AsIter<Item = Self::Selection> + Hash + Eq + Ord {
    type Selection: AbstractSelection;
}
