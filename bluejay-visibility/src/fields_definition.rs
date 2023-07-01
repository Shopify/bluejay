use crate::{Cache, FieldDefinition, Warden};
use bluejay_core::definition::{self, SchemaDefinition};
use bluejay_core::AsIter;
use once_cell::unsync::OnceCell;

pub struct FieldsDefinition<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::FieldsDefinition,
    cache: &'a Cache<'a, S, W>,
    fields_definition: OnceCell<Vec<FieldDefinition<'a, S, W>>>,
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> FieldsDefinition<'a, S, W> {
    pub(crate) fn new(inner: &'a S::FieldsDefinition, cache: &'a Cache<'a, S, W>) -> Self {
        Self {
            inner,
            cache,
            fields_definition: OnceCell::new(),
        }
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> AsIter
    for FieldsDefinition<'a, S, W>
{
    type Item = FieldDefinition<'a, S, W>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.fields_definition
            .get_or_init(|| {
                self.inner
                    .iter()
                    .filter_map(|fd| FieldDefinition::new(fd, self.cache))
                    .collect()
            })
            .iter()
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::FieldsDefinition
    for FieldsDefinition<'a, S, W>
{
    type FieldDefinition = FieldDefinition<'a, S, W>;
}
