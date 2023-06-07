use crate::definition::{Error, Rule, Visitor};
use crate::utils::duplicates;
use bluejay_core::definition::{InputObjectTypeDefinition, InputValueDefinition, SchemaDefinition};
use bluejay_core::AsIter;

pub struct InputFieldDefinitionUniqueness<'a, S: SchemaDefinition + 'a> {
    errors: Vec<Error<'a, S>>,
}

impl<'a, S: SchemaDefinition> Visitor<'a, S> for InputFieldDefinitionUniqueness<'a, S> {
    fn visit_input_object_type_definition(
        &mut self,
        input_object_type_definition: &'a <S as SchemaDefinition>::InputObjectTypeDefinition,
    ) {
        self.errors.extend(
            duplicates(
                input_object_type_definition
                    .input_field_definitions()
                    .iter(),
                InputValueDefinition::name,
            )
            .map(|(name, input_value_definitions)| {
                Error::NonUniqueInputValueDefinitionNames {
                    name,
                    input_value_definitions,
                }
            }),
        );
    }
}

impl<'a, S: SchemaDefinition> IntoIterator for InputFieldDefinitionUniqueness<'a, S> {
    type Item = Error<'a, S>;
    type IntoIter = std::vec::IntoIter<Error<'a, S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a, S: SchemaDefinition> Rule<'a, S> for InputFieldDefinitionUniqueness<'a, S> {
    type Error = Error<'a, S>;

    fn new(_: &'a S) -> Self {
        Self { errors: Vec::new() }
    }
}
