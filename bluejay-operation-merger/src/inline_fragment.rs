use crate::{Context, EmptyDirectives, MergedSelectionSet};
use bluejay_core::executable::{ExecutableDocument, InlineFragment};

pub struct MergedInlineFragment<'a, E: ExecutableDocument> {
    pub type_condition: &'a str,
    pub selection_set: MergedSelectionSet<'a, E>,
}

impl<'a, E: ExecutableDocument> InlineFragment for MergedInlineFragment<'a, E> {
    type Directives = EmptyDirectives<false, E>;
    type SelectionSet = MergedSelectionSet<'a, E>;

    fn type_condition(&self) -> Option<&str> {
        Some(self.type_condition)
    }

    fn selection_set(&self) -> &Self::SelectionSet {
        &self.selection_set
    }

    fn directives(&self) -> &Self::Directives {
        &EmptyDirectives::DEFAULT
    }
}

impl<'a, E: ExecutableDocument> MergedInlineFragment<'a, E> {
    pub(crate) fn new(type_condition: &'a str, context: &Context<'a, E>) -> Self {
        Self {
            type_condition,
            selection_set: MergedSelectionSet::new(context),
        }
    }

    pub(crate) fn selection_set_mut(&mut self) -> &mut MergedSelectionSet<'a, E> {
        &mut self.selection_set
    }
}
