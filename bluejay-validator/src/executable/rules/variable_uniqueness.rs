use crate::executable::{Cache, Error, Rule, Visitor};
use crate::utils::duplicates;
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::{ExecutableDocument, VariableDefinition};
use bluejay_core::AsIter;

pub struct VariableUniqueness<'a, E: ExecutableDocument, S: SchemaDefinition> {
    errors: Vec<Error<'a, E, S>>,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Visitor<'a, E, S>
    for VariableUniqueness<'a, E, S>
{
    fn new(_: &'a E, _: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self { errors: Vec::new() }
    }

    fn visit_variable_definitions(
        &mut self,
        variable_definitions: &'a <E as ExecutableDocument>::VariableDefinitions,
    ) {
        self.errors.extend(
            duplicates(variable_definitions.iter(), VariableDefinition::variable).map(
                |(name, variable_definitions)| Error::NonUniqueVariableDefinitionNames {
                    name,
                    variable_definitions,
                },
            ),
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
    type Error = Error<'a, E, S>;
}
