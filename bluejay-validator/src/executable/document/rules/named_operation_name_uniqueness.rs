use crate::executable::{
    document::{Error, Rule, Visitor},
    Cache,
};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::{
    ExecutableDocument, ExplicitOperationDefinition, OperationDefinition,
    OperationDefinitionReference,
};
use std::collections::BTreeMap;

pub struct NamedOperationNameUniqueness<'a, E: ExecutableDocument> {
    operations: BTreeMap<&'a str, Vec<&'a E::ExplicitOperationDefinition>>,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition> Visitor<'a, E, S>
    for NamedOperationNameUniqueness<'a, E>
{
    fn new(_: &'a E, _: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self {
            operations: BTreeMap::new(),
        }
    }

    fn visit_operation_definition(&mut self, operation_definition: &'a E::OperationDefinition) {
        if let OperationDefinitionReference::Explicit(eod) = operation_definition.as_ref() {
            if let Some(name) = eod.name() {
                self.operations.entry(name).or_default().push(eod);
            }
        }
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for NamedOperationNameUniqueness<'a, E>
{
    type Error = Error<'a, E, S>;
    type Errors = std::iter::FilterMap<
        std::collections::btree_map::IntoIter<&'a str, Vec<&'a E::ExplicitOperationDefinition>>,
        fn((&'a str, Vec<&'a E::ExplicitOperationDefinition>)) -> Option<Error<'a, E, S>>,
    >;

    fn into_errors(self) -> Self::Errors {
        self.operations
            .into_iter()
            .filter_map(|(name, operations)| {
                (operations.len() > 1)
                    .then_some(Error::NonUniqueOperationNames { name, operations })
            })
    }
}
