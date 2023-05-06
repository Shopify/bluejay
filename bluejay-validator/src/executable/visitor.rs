use crate::executable::Path;
use bluejay_core::definition::{
    DirectiveLocation, SchemaDefinition, TypeDefinitionReferenceFromAbstract,
};
use bluejay_core::executable::ExecutableDocument;

pub trait Visitor<'a, E: ExecutableDocument, S: SchemaDefinition> {
    fn visit_operation_definition(&mut self, _operation_definition: &'a E::OperationDefinition) {}

    fn visit_selection_set(
        &mut self,
        _selection_set: &'a E::SelectionSet,
        _type: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
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
        _scoped_type: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
    ) {
    }

    fn visit_fragment_spread(
        &mut self,
        _fragment_spread: &'a E::FragmentSpread,
        _scoped_type: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
        _path: &Path<'a, E>,
    ) {
    }

    fn visit_const_value(
        &mut self,
        _value: &'a E::Value<true>,
        _expected_type: &'a S::InputTypeReference,
    ) {
    }

    fn visit_variable_value(
        &mut self,
        _value: &'a E::Value<false>,
        _expected_type: &'a S::InputTypeReference,
        _path: &Path<'a, E>,
    ) {
    }

    fn visit_variable_definition(&mut self, _variable_definition: &'a E::VariableDefinition) {}

    fn visit_variable_definitions(&mut self, _variable_definitions: &'a E::VariableDefinitions) {}
}
