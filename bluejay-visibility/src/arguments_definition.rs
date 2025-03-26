use crate::{Cache, InputValueDefinition, Warden};
use bluejay_core::definition::{self, SchemaDefinition};
use bluejay_core::AsIter;
use once_cell::unsync::OnceCell;

pub struct ArgumentsDefinition<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::ArgumentsDefinition,
    cache: &'a Cache<'a, S, W>,
    arguments_definition: OnceCell<Vec<InputValueDefinition<'a, S, W>>>,
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> ArgumentsDefinition<'a, S, W> {
    pub(crate) fn new(inner: &'a S::ArgumentsDefinition, cache: &'a Cache<'a, S, W>) -> Self {
        Self {
            inner,
            cache,
            arguments_definition: OnceCell::new(),
        }
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> AsIter
    for ArgumentsDefinition<'a, S, W>
{
    type Item = InputValueDefinition<'a, S, W>;
    type Iterator<'b>
        = std::slice::Iter<'b, Self::Item>
    where
        'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.arguments_definition
            .get_or_init(|| {
                self.inner
                    .iter()
                    .filter_map(|ivd| InputValueDefinition::new(ivd, self.cache))
                    .collect()
            })
            .iter()
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::ArgumentsDefinition
    for ArgumentsDefinition<'a, S, W>
{
    type ArgumentDefinition = InputValueDefinition<'a, S, W>;
}
