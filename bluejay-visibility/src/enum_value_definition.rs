use crate::{Cache, Warden};
use bluejay_core::definition::{self, SchemaDefinition};
use std::marker::PhantomData;

pub struct EnumValueDefinition<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::EnumValueDefinition,
    warden: PhantomData<W>,
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
            })
    }

    pub fn inner(&self) -> &'a S::EnumValueDefinition {
        self.inner
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::EnumValueDefinition
    for EnumValueDefinition<'a, S, W>
{
    type Directives = <S::EnumValueDefinition as definition::EnumValueDefinition>::Directives;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.inner.directives()
    }
}
