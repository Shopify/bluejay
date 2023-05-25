use crate::definition::{Error, Rule, Visitor};
use bluejay_core::definition::{InputObjectTypeDefinition, InputValueDefinition, SchemaDefinition};
use bluejay_core::AsIter;
use std::collections::BTreeMap;

pub struct InputFieldDefinitionUniqueness<'a, S: SchemaDefinition + 'a> {
    errors: Vec<Error<'a, S>>,
}

impl<'a, S: SchemaDefinition> Visitor<'a, S> for InputFieldDefinitionUniqueness<'a, S> {
    fn visit_input_object_type_definition(
        &mut self,
        input_object_type_definition: &'a <S as SchemaDefinition>::InputObjectTypeDefinition,
    ) {
        let indexed = input_object_type_definition
            .input_field_definitions()
            .iter()
            .fold(
                BTreeMap::new(),
                |mut indexed: BTreeMap<&'a str, Vec<&'a S::InputValueDefinition>>,
                 ivd: &'a S::InputValueDefinition| {
                    indexed.entry(ivd.name()).or_default().push(ivd);
                    indexed
                },
            );

        self.errors.extend(
            indexed
                .into_iter()
                .filter_map(|(name, input_value_definitions)| {
                    (input_value_definitions.len() > 1).then_some(
                        Error::NonUniqueInputValueDefinitionNames {
                            name,
                            input_value_definitions,
                        },
                    )
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
