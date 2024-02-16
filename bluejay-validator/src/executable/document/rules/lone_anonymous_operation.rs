use crate::executable::{
    document::{Error, Rule, Visitor},
    Cache,
};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::{ExecutableDocument, OperationDefinition};

pub struct LoneAnonymousOperation<'a, E: ExecutableDocument> {
    anonymous_operations: Vec<&'a E::OperationDefinition>,
    executable_document: &'a E,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Visitor<'a, E, S>
    for LoneAnonymousOperation<'a, E>
{
    fn new(executable_document: &'a E, _: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self {
            anonymous_operations: Vec::new(),
            executable_document,
        }
    }

    fn visit_operation_definition(&mut self, operation_definition: &'a E::OperationDefinition) {
        if operation_definition.as_ref().name().is_none() {
            self.anonymous_operations.push(operation_definition);
        }
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for LoneAnonymousOperation<'a, E>
{
    type Error = Error<'a, E, S>;
    type Errors = <Option<Error<'a, E, S>> as IntoIterator>::IntoIter;

    fn into_errors(self) -> Self::Errors {
        (self.executable_document.operation_definitions().len() != 1
            && !self.anonymous_operations.is_empty())
        .then_some(Error::NotLoneAnonymousOperation {
            anonymous_operations: self.anonymous_operations,
        })
        .into_iter()
    }
}
