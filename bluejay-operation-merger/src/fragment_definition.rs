use crate::{EmptyDirectives, MergedSelectionSet, Never};
use bluejay_core::{executable::FragmentDefinition, Indexable};

/// This is never instantiated because we will always inline fragment definitions in the merged document.
/// But to conform to the core traits, we need to provide a type that implements `FragmentDefinition`.
pub struct MergedFragmentDefinition<'a> {
    name: &'a str,
    type_condition: &'a str,
    selection_set: MergedSelectionSet<'a>,
    /// This field is never used, but its presence ensures this will never be instantiated
    _never: Never,
}

impl<'a> FragmentDefinition for MergedFragmentDefinition<'a> {
    type SelectionSet = MergedSelectionSet<'a>;
    type Directives = EmptyDirectives<'a>;

    fn name(&self) -> &str {
        self.name
    }

    fn selection_set(&self) -> &Self::SelectionSet {
        &self.selection_set
    }

    fn type_condition(&self) -> &str {
        self.type_condition
    }

    fn directives(&self) -> &Self::Directives {
        &EmptyDirectives::DEFAULT
    }
}

impl<'a> Indexable for MergedFragmentDefinition<'a> {
    type Id = &'a str;

    fn id(&self) -> &Self::Id {
        &self.name
    }
}
