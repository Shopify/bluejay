use crate::{ArgumentsDefinition, Cache, Warden};
use bluejay_core::definition::{self, SchemaDefinition};
use once_cell::unsync::OnceCell;

pub struct DirectiveDefinition<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::DirectiveDefinition,
    cache: &'a Cache<'a, S, W>,
    arguments_definition: OnceCell<Option<ArgumentsDefinition<'a, S, W>>>,
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> DirectiveDefinition<'a, S, W> {
    pub fn new(inner: &'a S::DirectiveDefinition, cache: &'a Cache<'a, S, W>) -> Option<Self> {
        cache
            .warden()
            .is_directive_definition_visible(inner)
            .then(|| Self {
                inner,
                cache,
                arguments_definition: OnceCell::new(),
            })
    }

    pub fn inner(&self) -> &'a S::DirectiveDefinition {
        self.inner
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::DirectiveDefinition
    for DirectiveDefinition<'a, S, W>
{
    type ArgumentsDefinition = ArgumentsDefinition<'a, S, W>;
    type DirectiveLocations =
        <S::DirectiveDefinition as definition::DirectiveDefinition>::DirectiveLocations;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn is_builtin(&self) -> bool {
        self.inner.is_builtin()
    }

    fn arguments_definition(&self) -> Option<&Self::ArgumentsDefinition> {
        self.arguments_definition
            .get_or_init(|| {
                self.inner
                    .arguments_definition()
                    .map(|ad| ArgumentsDefinition::new(ad, self.cache))
            })
            .as_ref()
    }

    fn is_repeatable(&self) -> bool {
        self.inner.is_repeatable()
    }

    fn locations(&self) -> &Self::DirectiveLocations {
        self.inner.locations()
    }
}
