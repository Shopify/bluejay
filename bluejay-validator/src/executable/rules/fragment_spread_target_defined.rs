use crate::executable::{Cache, Error, Rule, Visitor};
use bluejay_core::definition::{SchemaDefinition, TypeDefinitionReferenceFromAbstract};
use bluejay_core::executable::{ExecutableDocument, FragmentDefinition, FragmentSpread};
use std::collections::HashSet;

pub struct FragmentSpreadTargetDefined<'a, E: ExecutableDocument, S: SchemaDefinition> {
    errors: Vec<Error<'a, E, S>>,
    fragment_definition_names: HashSet<&'a str>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Visitor<'a, E, S>
    for FragmentSpreadTargetDefined<'a, E, S>
{
    fn visit_fragment_spread(
        &mut self,
        fragment_spread: &'a <E as ExecutableDocument>::FragmentSpread,
        _scoped_type: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
    ) {
        if !self
            .fragment_definition_names
            .contains(fragment_spread.name())
        {
            self.errors
                .push(Error::FragmentSpreadTargetUndefined { fragment_spread });
        }
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> IntoIterator
    for FragmentSpreadTargetDefined<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for FragmentSpreadTargetDefined<'a, E, S>
{
    fn new(executable_document: &'a E, _: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self {
            errors: Vec::new(),
            fragment_definition_names: HashSet::from_iter(
                executable_document
                    .fragment_definitions()
                    .iter()
                    .map(FragmentDefinition::name),
            ),
        }
    }
}
