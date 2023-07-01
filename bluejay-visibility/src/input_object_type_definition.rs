use crate::{Cache, InputFieldsDefinition, Warden};
use bluejay_core::definition::{self, SchemaDefinition};
use once_cell::unsync::OnceCell;

pub struct InputObjectTypeDefinition<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::InputObjectTypeDefinition,
    cache: &'a Cache<'a, S, W>,
    input_fields_definition: OnceCell<InputFieldsDefinition<'a, S, W>>,
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>>
    InputObjectTypeDefinition<'a, S, W>
{
    pub fn new(inner: &'a S::InputObjectTypeDefinition, cache: &'a Cache<'a, S, W>) -> Self {
        Self {
            inner,
            cache,
            input_fields_definition: OnceCell::new(),
        }
    }

    pub fn inner(&self) -> &'a S::InputObjectTypeDefinition {
        self.inner
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>>
    definition::InputObjectTypeDefinition for InputObjectTypeDefinition<'a, S, W>
{
    type Directives =
        <S::InputObjectTypeDefinition as definition::InputObjectTypeDefinition>::Directives;
    type InputFieldsDefinition = InputFieldsDefinition<'a, S, W>;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.inner.directives()
    }

    fn input_field_definitions(&self) -> &Self::InputFieldsDefinition {
        self.input_fields_definition.get_or_init(|| {
            InputFieldsDefinition::new(self.inner.input_field_definitions(), self.cache)
        })
    }
}
