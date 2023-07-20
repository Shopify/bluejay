use crate::{Cache, Directives, Warden};
use bluejay_core::definition::{self, SchemaDefinition};

pub struct ScalarTypeDefinition<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::CustomScalarTypeDefinition,
    directives: Option<Directives<'a, S, W>>,
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> ScalarTypeDefinition<'a, S, W> {
    pub(crate) fn new(
        inner: &'a S::CustomScalarTypeDefinition,
        cache: &'a Cache<'a, S, W>,
    ) -> Self {
        Self {
            inner,
            directives: definition::ScalarTypeDefinition::directives(inner)
                .map(|d| Directives::new(d, cache)),
        }
    }

    pub fn inner(&self) -> &'a S::CustomScalarTypeDefinition {
        self.inner
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::ScalarTypeDefinition
    for ScalarTypeDefinition<'a, S, W>
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

    fn coerce_input<const CONST: bool>(
        &self,
        value: &impl bluejay_core::Value<CONST>,
    ) -> Result<(), std::borrow::Cow<'static, str>> {
        self.inner.coerce_input(value)
    }
}
