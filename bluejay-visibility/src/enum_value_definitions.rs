use crate::{Cache, EnumValueDefinition, Warden};
use bluejay_core::definition::{self, SchemaDefinition};
use bluejay_core::AsIter;
use once_cell::unsync::OnceCell;

pub struct EnumValueDefinitions<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::EnumValueDefinitions,
    cache: &'a Cache<'a, S, W>,
    enum_value_definitions: OnceCell<Vec<EnumValueDefinition<'a, S, W>>>,
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> EnumValueDefinitions<'a, S, W> {
    pub(crate) fn new(inner: &'a S::EnumValueDefinitions, cache: &'a Cache<'a, S, W>) -> Self {
        Self {
            inner,
            cache,
            enum_value_definitions: OnceCell::new(),
        }
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> AsIter
    for EnumValueDefinitions<'a, S, W>
{
    type Item = EnumValueDefinition<'a, S, W>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.enum_value_definitions
            .get_or_init(|| {
                self.inner
                    .iter()
                    .filter_map(|evd| EnumValueDefinition::new(evd, self.cache))
                    .collect()
            })
            .iter()
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::EnumValueDefinitions
    for EnumValueDefinitions<'a, S, W>
{
    type EnumValueDefinition = EnumValueDefinition<'a, S, W>;
}
