use crate::executable::{Cache, Error, Rule, Visitor};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::{ExecutableDocument, OperationDefinition};
use std::marker::PhantomData;

pub struct LoneAnonymousOperation<'a, E: ExecutableDocument, S: SchemaDefinition> {
    anonymous_operations: Vec<&'a E::OperationDefinition>,
    executable_document: &'a E,
    schema_definition: PhantomData<S>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Visitor<'a, E, S>
    for LoneAnonymousOperation<'a, E, S>
{
    fn visit_operation_definition(&mut self, operation_definition: &'a E::OperationDefinition) {
        if operation_definition.as_ref().name().is_none() {
            self.anonymous_operations.push(operation_definition);
        }
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> IntoIterator
    for LoneAnonymousOperation<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = <Option<Error<'a, E, S>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (self.executable_document.operation_definitions().len() != 1
            && !self.anonymous_operations.is_empty())
        .then_some(Error::NotLoneAnonymousOperation {
            anonymous_operations: self.anonymous_operations,
        })
        .into_iter()
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for LoneAnonymousOperation<'a, E, S>
{
    type Error = Error<'a, E, S>;

    fn new(executable_document: &'a E, _: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self {
            anonymous_operations: Vec::new(),
            executable_document,
            schema_definition: Default::default(),
        }
    }
}
