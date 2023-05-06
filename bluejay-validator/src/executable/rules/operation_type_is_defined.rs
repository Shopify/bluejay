use crate::executable::{Cache, Error, Rule, Visitor};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::{
    AbstractOperationDefinition, ExecutableDocument, ExplicitOperationDefinition,
    OperationDefinition,
};
use bluejay_core::OperationType;

pub struct OperationTypeIsDefined<'a, E: ExecutableDocument, S: SchemaDefinition> {
    errors: Vec<Error<'a, E, S>>,
    schema_definition: &'a S,
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition> Visitor<'a, E, S>
    for OperationTypeIsDefined<'a, E, S>
{
    fn visit_operation_definition(&mut self, operation_definition: &'a E::OperationDefinition) {
        if let OperationDefinition::Explicit(eod) = operation_definition.as_ref() {
            match eod.operation_type() {
                OperationType::Query => {}
                OperationType::Mutation if self.schema_definition.mutation().is_some() => {}
                OperationType::Subscription if self.schema_definition.subscription().is_some() => {}
                _ => {
                    self.errors
                        .push(Error::OperationTypeNotDefined { operation: eod });
                }
            }
        }
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> IntoIterator
    for OperationTypeIsDefined<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = <Vec<Error<'a, E, S>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for OperationTypeIsDefined<'a, E, S>
{
    fn new(_: &'a E, schema_definition: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self {
            errors: Vec::new(),
            schema_definition,
        }
    }
}
