use crate::executable::Selection;
use crate::{AsIter, Indexable};

pub trait SelectionSet: AsIter<Item = Self::Selection> + Indexable {
    type Selection: Selection;
}
