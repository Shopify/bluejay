use crate::{Context, EmptyDirectives, MergedArguments, MergedSelectionSet};
use bluejay_core::executable::{ExecutableDocument, Field};
use std::borrow::Cow;

pub struct MergedField<'a> {
    name: &'a str,
    alias: Option<Cow<'a, str>>,
    arguments: Option<MergedArguments<'a, false>>,
    selection_set: Option<MergedSelectionSet<'a>>,
}

impl<'a> Field for MergedField<'a> {
    type Arguments = MergedArguments<'a, false>;
    type Directives = EmptyDirectives<'a>;
    type SelectionSet = MergedSelectionSet<'a>;

    fn alias(&self) -> Option<&str> {
        self.alias.as_deref()
    }

    fn name(&self) -> &str {
        self.name
    }

    fn arguments(&self) -> Option<&Self::Arguments> {
        self.arguments.as_ref()
    }

    fn directives(&self) -> &Self::Directives {
        &EmptyDirectives::DEFAULT
    }

    fn selection_set(&self) -> Option<&Self::SelectionSet> {
        self.selection_set.as_ref()
    }
}

impl<'a> MergedField<'a> {
    pub(crate) fn new<E: ExecutableDocument>(
        name: &'a str,
        alias: Option<Cow<'a, str>>,
        arguments: Option<&'a E::Arguments<false>>,
        context: &Context<'a, E>,
    ) -> Self {
        Self {
            name,
            alias,
            arguments: arguments.map(|arguments| MergedArguments::new(arguments, context)),
            selection_set: None,
        }
    }

    pub(crate) fn selection_set_mut(&mut self) -> &mut Option<MergedSelectionSet<'a>> {
        &mut self.selection_set
    }

    /// This method is added in addition to the `bluejay_core::executable::Field` method
    /// of the same name to allow getting a reference with lifetime `'a`.
    pub(crate) fn arguments(&self) -> Option<&MergedArguments<'a, false>> {
        self.arguments.as_ref()
    }
}
