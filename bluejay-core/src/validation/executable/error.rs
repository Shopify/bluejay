use crate::executable::{
    ExecutableDocument,
    OperationDefinitionFromExecutableDocument,
};
use crate::definition::{
    SchemaDefinition,
    TypeDefinitionReferenceFromAbstract,
};

pub enum Error<'a, E: ExecutableDocument<'a>, S: SchemaDefinition<'a>> {
    NonUniqueOperationNames { name: &'a str, operations: Vec<&'a E::ExplicitOperationDefinition> },
    NotLoneAnonymousOperation { anonymous_operations: Vec<&'a OperationDefinitionFromExecutableDocument<'a, E>> },
    SubscriptionRootNotSingleField { operation: &'a OperationDefinitionFromExecutableDocument<'a, E> },
    FieldDoesNotExistOnType { field: &'a E::Field, r#type: &'a TypeDefinitionReferenceFromAbstract<S::TypeDefinitionReference> },
    FieldSelectionsDoNotMerge { selection_set: &'a E::SelectionSet },
}
