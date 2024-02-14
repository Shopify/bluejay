use crate::executable::{operation::VariableValues, Cache};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::ExecutableDocument;

pub trait Visitor<'a, E: ExecutableDocument, S: SchemaDefinition, V: VariableValues> {
    fn new(
        operation_definition: &'a E::OperationDefinition,
        schema_definition: &'a S,
        variable_values: &'a V,
        cache: &'a Cache<'a, E, S>,
    ) -> Self;

    /// Visits the field. If a field is part of a fragment definition, it will be visited
    /// every time the fragment is spread.
    /// `included` is true when the field is known to be included in the response
    /// (based on the usage of `@include` and `@skip` directives and the variable values).
    fn visit_field(
        &mut self,
        _field: &'a E::Field,
        _field_definition: &'a S::FieldDefinition,
        _included: bool,
    ) {
    }

    /// Called after the field and all of its children have been visited.
    /// `included` is true when the field is known to be included in the response
    /// (based on the usage of `@include` and `@skip` directives and the variable values).
    fn leave_field(
        &mut self,
        _field: &'a <E as ExecutableDocument>::Field,
        _field_definition: &'a S::FieldDefinition,
        _included: bool,
    ) {
    }
}
