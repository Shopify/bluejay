use crate::executable::{Cache, Error, Rule, Visitor};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::{
    ExecutableDocument, ExplicitOperationDefinition, OperationDefinition,
    OperationDefinitionFromExecutableDocument,
};
use std::collections::HashMap;
use std::marker::PhantomData;

pub struct NamedOperationNameUniqueness<'a, E: ExecutableDocument, S: SchemaDefinition> {
    operations: HashMap<&'a str, Vec<&'a E::ExplicitOperationDefinition>>,
    schema_definition: PhantomData<S>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Visitor<'a, E, S>
    for NamedOperationNameUniqueness<'a, E, S>
{
    fn visit_operation_definition(
        &mut self,
        operation_definition: &'a OperationDefinitionFromExecutableDocument<E>,
    ) {
        if let OperationDefinition::Explicit(eod) = operation_definition {
            if let Some(name) = eod.name() {
                self.operations.entry(name).or_default().push(eod);
            }
        }
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> IntoIterator
    for NamedOperationNameUniqueness<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = std::iter::FilterMap<
        std::collections::hash_map::IntoIter<&'a str, Vec<&'a E::ExplicitOperationDefinition>>,
        fn((&'a str, Vec<&'a E::ExplicitOperationDefinition>)) -> Option<Error<'a, E, S>>,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.operations
            .into_iter()
            .filter_map(|(name, operations)| {
                (operations.len() > 1)
                    .then_some(Error::NonUniqueOperationNames { name, operations })
            })
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for NamedOperationNameUniqueness<'a, E, S>
{
    fn new(_: &'a E, _: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self {
            operations: HashMap::new(),
            schema_definition: Default::default(),
        }
    }
}
