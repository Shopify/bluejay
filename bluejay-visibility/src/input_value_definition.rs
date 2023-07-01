use crate::{Cache, InputType, Warden};
use bluejay_core::definition::{self, SchemaDefinition};
use once_cell::unsync::OnceCell;

pub struct InputValueDefinition<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::InputValueDefinition,
    cache: &'a Cache<'a, S, W>,
    r#type: OnceCell<InputType<'a, S, W>>,
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> InputValueDefinition<'a, S, W> {
    pub(crate) fn new(
        inner: &'a S::InputValueDefinition,
        cache: &'a Cache<'a, S, W>,
    ) -> Option<Self> {
        cache
            .warden()
            .is_input_value_definition_visible(inner)
            .then_some(Self {
                inner,
                cache,
                r#type: OnceCell::new(),
            })
    }

    pub fn inner(&self) -> &'a S::InputValueDefinition {
        self.inner
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::InputValueDefinition
    for InputValueDefinition<'a, S, W>
{
    type Value = <S::InputValueDefinition as definition::InputValueDefinition>::Value;
    type Directives = <S::InputValueDefinition as definition::InputValueDefinition>::Directives;
    type InputType = InputType<'a, S, W>;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn default_value(&self) -> Option<&Self::Value> {
        self.inner.default_value()
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.inner.directives()
    }

    fn r#type(&self) -> &Self::InputType {
        self.r#type
            .get_or_init(|| InputType::new(self.inner.r#type(), self.cache))
    }
}
