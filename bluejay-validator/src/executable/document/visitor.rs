use crate::executable::{document::Path, Cache};
use bluejay_core::definition::{DirectiveLocation, SchemaDefinition, TypeDefinitionReference};
use bluejay_core::executable::ExecutableDocument;

pub trait Visitor<'a, E: ExecutableDocument, S: SchemaDefinition> {
    fn new(
        executable_document: &'a E,
        schema_definition: &'a S,
        cache: &'a Cache<'a, E, S>,
    ) -> Self;

    fn visit_operation_definition(&mut self, _operation_definition: &'a E::OperationDefinition) {}

    fn visit_selection_set(
        &mut self,
        _selection_set: &'a E::SelectionSet,
        _type: TypeDefinitionReference<'a, S::TypeDefinition>,
    ) {
    }

    fn visit_field(
        &mut self,
        _field: &'a E::Field,
        _field_definition: &'a S::FieldDefinition,
        _path: &Path<'a, E>,
    ) {
    }

    fn visit_const_directive(
        &mut self,
        _directive: &'a E::Directive<true>,
        _location: DirectiveLocation,
    ) {
    }

    fn visit_variable_directive(
        &mut self,
        _directive: &'a E::Directive<false>,
        _location: DirectiveLocation,
    ) {
    }

    fn visit_const_directives(
        &mut self,
        _directives: &'a E::Directives<true>,
        _location: DirectiveLocation,
    ) {
    }

    fn visit_variable_directives(
        &mut self,
        _directives: &'a E::Directives<false>,
        _location: DirectiveLocation,
    ) {
    }

    fn visit_fragment_definition(&mut self, _fragment_definition: &'a E::FragmentDefinition) {}

    fn visit_inline_fragment(
        &mut self,
        _inline_fragment: &'a E::InlineFragment,
        _scoped_type: TypeDefinitionReference<'a, S::TypeDefinition>,
    ) {
    }

    fn visit_fragment_spread(
        &mut self,
        _fragment_spread: &'a E::FragmentSpread,
        _scoped_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        _path: &Path<'a, E>,
    ) {
    }

    fn visit_const_argument(
        &mut self,
        _argument: &'a E::Argument<true>,
        _input_value_definition: &'a S::InputValueDefinition,
    ) {
    }

    fn visit_variable_argument(
        &mut self,
        _argument: &'a E::Argument<false>,
        _input_value_definition: &'a S::InputValueDefinition,
        _path: &Path<'a, E>,
    ) {
    }

    fn visit_variable_definition(&mut self, _variable_definition: &'a E::VariableDefinition) {}

    fn visit_variable_definitions(&mut self, _variable_definitions: &'a E::VariableDefinitions) {}
}
