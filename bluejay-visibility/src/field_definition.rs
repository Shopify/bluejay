use crate::{ArgumentsDefinition, Cache, Directives, OutputType, Warden};
use bluejay_core::definition::{self, SchemaDefinition};
use once_cell::unsync::OnceCell;

pub struct FieldDefinition<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::FieldDefinition,
    cache: &'a Cache<'a, S, W>,
    r#type: OutputType<'a, S, W>,
    arguments_definition: OnceCell<Option<ArgumentsDefinition<'a, S, W>>>,
    directives: Option<Directives<'a, S, W>>,
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> FieldDefinition<'a, S, W> {
    pub(crate) fn new(inner: &'a S::FieldDefinition, cache: &'a Cache<'a, S, W>) -> Option<Self> {
        cache
            .warden()
            .is_field_definition_visible(inner)
            .then(|| {
                OutputType::new(definition::FieldDefinition::r#type(inner), cache).map(|r#type| {
                    Self {
                        inner,
                        cache,
                        r#type,
                        arguments_definition: OnceCell::new(),
                        directives: definition::FieldDefinition::directives(inner)
                            .map(|d| Directives::new(d, cache)),
                    }
                })
            })
            .flatten()
    }

    pub fn inner(&self) -> &'a S::FieldDefinition {
        self.inner
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::FieldDefinition
    for FieldDefinition<'a, S, W>
{
    type OutputType = OutputType<'a, S, W>;
    type Directives = Directives<'a, S, W>;
    type ArgumentsDefinition = ArgumentsDefinition<'a, S, W>;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn is_builtin(&self) -> bool {
        self.inner.is_builtin()
    }

    fn r#type(&self) -> &Self::OutputType {
        &self.r#type
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
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
}
