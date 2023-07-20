use crate::{Cache, Directives, Warden};
use bluejay_core::definition::{self, SchemaDefinition};
use std::marker::PhantomData;

pub struct EnumValueDefinition<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::EnumValueDefinition,
    warden: PhantomData<W>,
    directives: Option<Directives<'a, S, W>>,
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> EnumValueDefinition<'a, S, W> {
    pub(crate) fn new(
        inner: &'a S::EnumValueDefinition,
        cache: &'a Cache<'a, S, W>,
    ) -> Option<Self> {
        cache
            .warden()
            .is_enum_value_definition_visible(inner)
            .then_some(Self {
                inner,
                warden: Default::default(),
                directives: definition::EnumValueDefinition::directives(inner)
                    .map(|d| Directives::new(d, cache)),
            })
    }

    pub fn inner(&self) -> &'a S::EnumValueDefinition {
        self.inner
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::EnumValueDefinition
    for EnumValueDefinition<'a, S, W>
{
    type Directives = Directives<'a, S, W>;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}
