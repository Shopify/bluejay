use crate::{DirectiveDefinition, Warden};
use bluejay_core::definition::{Directive as CoreDefinitionDirective, SchemaDefinition};
use bluejay_core::Directive as CoreDirective;

pub struct Directive<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::Directive,
    definition: &'a DirectiveDefinition<'a, S, W>,
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> Directive<'a, S, W> {
    pub(crate) fn new(
        inner: &'a S::Directive,
        definition: &'a DirectiveDefinition<'a, S, W>,
    ) -> Self {
        Self { inner, definition }
    }
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> CoreDirective<true>
    for Directive<'a, S, W>
{
    type Arguments = <S::Directive as CoreDirective<true>>::Arguments;

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn arguments(&self) -> Option<&Self::Arguments> {
        self.inner.arguments()
    }
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> CoreDefinitionDirective
    for Directive<'a, S, W>
{
    type DirectiveDefinition = DirectiveDefinition<'a, S, W>;

    fn definition(&self) -> &Self::DirectiveDefinition {
        self.definition
    }
}
