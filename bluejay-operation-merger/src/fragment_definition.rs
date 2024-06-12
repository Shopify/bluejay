use crate::{EmptyDirectives, MergedSelectionSet, Never};
use bluejay_core::{
    executable::{ExecutableDocument, FragmentDefinition},
    Indexable,
};

/// This is never instantiated because we will always inline fragment definitions in the merged document.
/// But to conform to the core traits, we need to provide a type that implements `FragmentDefinition`.
pub struct MergedFragmentDefinition<'a, E: ExecutableDocument> {
    name: &'a str,
    type_condition: &'a str,
    selection_set: MergedSelectionSet<'a, E>,
    /// This field is never used, but its presence ensures this will never be instantiated
    _never: Never,
}

impl<'a, E: ExecutableDocument> FragmentDefinition for MergedFragmentDefinition<'a, E> {
    type SelectionSet = MergedSelectionSet<'a, E>;
    type Directives = EmptyDirectives<false, E>;

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

impl<'a, E: ExecutableDocument> Indexable for MergedFragmentDefinition<'a, E> {
    type Id = &'a str;

    fn id(&self) -> &Self::Id {
        &self.name
    }
}
