use crate::executable::{Cache, Error, Rule, Visitor};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::{ExecutableDocument, VariableDefinition};
use bluejay_core::AsIter;
use std::collections::BTreeMap;

pub struct VariableUniqueness<'a, E: ExecutableDocument, S: SchemaDefinition> {
    errors: Vec<Error<'a, E, S>>,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Visitor<'a, E, S>
    for VariableUniqueness<'a, E, S>
{
    fn visit_variable_definitions(
        &mut self,
        variable_definitions: &'a <E as ExecutableDocument>::VariableDefinitions,
    ) {
        let indexed: BTreeMap<&'a str, Vec<&'a E::VariableDefinition>> = variable_definitions
            .iter()
            .fold(BTreeMap::new(), |mut indexed, variable_definition| {
                indexed
                    .entry(variable_definition.variable())
                    .or_default()
                    .push(variable_definition);
                indexed
            });

        self.errors.extend(
            indexed
                .into_iter()
                .filter_map(|(name, variable_definitions)| {
                    (variable_definitions.len() > 1).then_some(
                        Error::NonUniqueVariableDefinitionNames {
                            name,
                            variable_definitions,
                        },
                    )
                }),
        );
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> IntoIterator
    for VariableUniqueness<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for VariableUniqueness<'a, E, S>
{
    fn new(_: &'a E, _: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self { errors: Vec::new() }
    }
}
