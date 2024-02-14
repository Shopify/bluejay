use crate::executable::{
    document::{Error, Path, Rule, Visitor},
    Cache,
};
use bluejay_core::definition::{SchemaDefinition, TypeDefinitionReference};
use bluejay_core::executable::{ExecutableDocument, FragmentDefinition, FragmentSpread};
use std::collections::HashSet;

pub struct FragmentSpreadTargetDefined<'a, E: ExecutableDocument, S: SchemaDefinition> {
    errors: Vec<Error<'a, E, S>>,
    fragment_definition_names: HashSet<&'a str>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Visitor<'a, E, S>
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

    fn visit_fragment_spread(
        &mut self,
        fragment_spread: &'a <E as ExecutableDocument>::FragmentSpread,
        _scoped_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        _path: &Path<'a, E>,
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

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for FragmentSpreadTargetDefined<'a, E, S>
{
    type Error = Error<'a, E, S>;
    type Errors = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_errors(self) -> Self::Errors {
        self.errors.into_iter()
    }
}
