use crate::{ArgumentsDefinition, Cache, OutputType, Warden};
use bluejay_core::definition::{self, SchemaDefinition};
use once_cell::unsync::OnceCell;

pub struct FieldDefinition<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::FieldDefinition,
    cache: &'a Cache<'a, S, W>,
    r#type: OnceCell<OutputType<'a, S, W>>,
    arguments_definition: OnceCell<Option<ArgumentsDefinition<'a, S, W>>>,
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> FieldDefinition<'a, S, W> {
    pub(crate) fn new(inner: &'a S::FieldDefinition, cache: &'a Cache<'a, S, W>) -> Option<Self> {
        cache
            .warden()
            .is_field_definition_visible(inner)
            .then_some(Self {
                inner,
                cache,
                r#type: OnceCell::new(),
                arguments_definition: OnceCell::new(),
            })
    }

    pub fn inner(&self) -> &'a S::FieldDefinition {
        self.inner
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::FieldDefinition
    for FieldDefinition<'a, S, W>
{
    type OutputType = OutputType<'a, S, W>;
    type Directives = <S::FieldDefinition as definition::FieldDefinition>::Directives;
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
        self.r#type
            .get_or_init(|| OutputType::new(self.inner.r#type(), self.cache))
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.inner.directives()
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
