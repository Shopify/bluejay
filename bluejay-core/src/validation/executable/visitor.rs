use crate::definition::{SchemaDefinition, TypeDefinitionReferenceFromAbstract};
use crate::executable::{ExecutableDocument, OperationDefinitionFromExecutableDocument};

pub trait Visitor<'a, E: ExecutableDocument<'a>, S: SchemaDefinition<'a>> {
    fn visit_operation(
        &mut self,
        _operation_definition: &'a OperationDefinitionFromExecutableDocument<'a, E>,
    ) {
    }
    fn visit_selection_set(
        &mut self,
        _selection_set: &'a E::SelectionSet,
        _type: &'a TypeDefinitionReferenceFromAbstract<S::TypeDefinitionReference>,
    ) {
    }
}
