use crate::executable::SelectionSet;
use crate::VariableDirectives;
use std::cmp::{Eq, Ord};
use std::hash::Hash;

pub trait FragmentDefinition: Eq + Hash + Ord {
    type Directives: VariableDirectives;
    type SelectionSet: SelectionSet;

    fn name(&self) -> &str;
    fn type_condition(&self) -> &str;
    fn directives(&self) -> &Self::Directives;
    fn selection_set(&self) -> &Self::SelectionSet;
}
