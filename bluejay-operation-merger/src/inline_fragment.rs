use crate::{Context, EmptyDirectives, MergedSelectionSet};
use bluejay_core::executable::{ExecutableDocument, InlineFragment};

pub struct MergedInlineFragment<'a> {
    pub type_condition: &'a str,
    pub selection_set: MergedSelectionSet<'a>,
}

impl<'a> InlineFragment for MergedInlineFragment<'a> {
    type Directives = EmptyDirectives<'a>;
    type SelectionSet = MergedSelectionSet<'a>;

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

impl<'a> MergedInlineFragment<'a> {
    pub(crate) fn new<E: ExecutableDocument>(
        type_condition: &'a str,
        context: &Context<'a, E>,
    ) -> Self {
        Self {
            type_condition,
            selection_set: MergedSelectionSet::new(context),
        }
    }

    pub(crate) fn selection_set_mut(&mut self) -> &mut MergedSelectionSet<'a> {
        &mut self.selection_set
    }
}
