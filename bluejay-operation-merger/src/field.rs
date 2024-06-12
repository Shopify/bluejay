use crate::{EmptyDirectives, MergedSelectionSet};
use bluejay_core::executable::{ExecutableDocument, Field};

pub struct MergedField<'a, E: ExecutableDocument> {
    name: &'a str,
    alias: Option<&'a str>,
    arguments: Option<&'a E::Arguments<false>>,
    selection_set: Option<MergedSelectionSet<'a, E>>,
}

impl<'a, E: ExecutableDocument> Field for MergedField<'a, E> {
    type Arguments = E::Arguments<false>;
    type Directives = EmptyDirectives<false, E>;
    type SelectionSet = MergedSelectionSet<'a, E>;

    fn alias(&self) -> Option<&str> {
        self.alias
    }

    fn name(&self) -> &str {
        self.name
    }

    fn arguments(&self) -> Option<&Self::Arguments> {
        self.arguments
    }

    fn directives(&self) -> &Self::Directives {
        &EmptyDirectives::DEFAULT
    }

    fn selection_set(&self) -> Option<&Self::SelectionSet> {
        self.selection_set.as_ref()
    }
}

impl<'a, E: ExecutableDocument> MergedField<'a, E> {
    pub(crate) fn new(
        name: &'a str,
        alias: Option<&'a str>,
        arguments: Option<&'a E::Arguments<false>>,
    ) -> Self {
        Self {
            name,
            alias,
            arguments,
            selection_set: None,
        }
    }

    pub(crate) fn selection_set_mut(&mut self) -> &mut Option<MergedSelectionSet<'a, E>> {
        &mut self.selection_set
    }

    /// This method is added in addition to the `bluejay_core::executable::Field` method
    /// of the same name to allow getting a reference with lifetime `'a`.
    pub(crate) fn arguments(&self) -> Option<&'a E::Arguments<false>> {
        self.arguments
    }
}
