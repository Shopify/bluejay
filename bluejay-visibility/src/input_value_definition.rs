use crate::{Cache, Directives, InputType, Warden};
use bluejay_core::definition::{self, HasDirectives, SchemaDefinition};

pub struct InputValueDefinition<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::InputValueDefinition,
    r#type: InputType<'a, S, W>,
    directives: Option<Directives<'a, S, W>>,
    cache: &'a Cache<'a, S, W>,
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> InputValueDefinition<'a, S, W> {
    pub(crate) fn new(
        inner: &'a S::InputValueDefinition,
        cache: &'a Cache<'a, S, W>,
    ) -> Option<Self> {
        cache
            .warden()
            .is_input_value_definition_visible(inner)
            .then(|| {
                InputType::new(definition::InputValueDefinition::r#type(inner), cache).map(
                    |r#type| Self {
                        inner,
                        r#type,
                        directives: inner.directives().map(|d| Directives::new(d, cache)),
                        cache,
                    },
                )
            })
            .flatten()
    }

    pub fn inner(&self) -> &'a S::InputValueDefinition {
        self.inner
    }

    pub fn cache(&self) -> &'a Cache<'a, S, W> {
        self.cache
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::InputValueDefinition
    for InputValueDefinition<'a, S, W>
{
    type Value = <S::InputValueDefinition as definition::InputValueDefinition>::Value;
    type InputType = InputType<'a, S, W>;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn default_value(&self) -> Option<&Self::Value> {
        self.cache
            .warden()
            .input_value_definition_default_value(self)
    }

    fn r#type(&self) -> &Self::InputType {
        &self.r#type
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> HasDirectives
    for InputValueDefinition<'a, S, W>
{
    type Directives = Directives<'a, S, W>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}
