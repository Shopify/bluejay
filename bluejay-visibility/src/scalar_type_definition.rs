use crate::{Cache, Directives, Warden};
use bluejay_core::definition::{self, HasDirectives, SchemaDefinition};

pub struct ScalarTypeDefinition<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::CustomScalarTypeDefinition,
    directives: Option<Directives<'a, S, W>>,
    cache: &'a Cache<'a, S, W>,
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> ScalarTypeDefinition<'a, S, W> {
    pub(crate) fn new(
        inner: &'a S::CustomScalarTypeDefinition,
        cache: &'a Cache<'a, S, W>,
    ) -> Self {
        Self {
            inner,
            directives: inner.directives().map(|d| Directives::new(d, cache)),
            cache,
        }
    }

    pub fn inner(&self) -> &'a S::CustomScalarTypeDefinition {
        self.inner
    }
}

impl<S: SchemaDefinition, W: Warden<SchemaDefinition = S>> definition::ScalarTypeDefinition
    for ScalarTypeDefinition<'_, S, W>
{
    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn coerce_input<const CONST: bool>(
        &self,
        value: &impl bluejay_core::Value<CONST>,
    ) -> Result<(), std::borrow::Cow<'static, str>> {
        self.cache
            .warden()
            .custom_scalar_definition_coerce_input(self.inner, value)
    }
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> HasDirectives
    for ScalarTypeDefinition<'a, S, W>
{
    type Directives = Directives<'a, S, W>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}
