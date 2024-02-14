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

macro_rules! impl_visitor {
    ($n:literal) => {
        seq_macro::seq!(N in 0..$n {
            impl<'a, E: ExecutableDocument, S: SchemaDefinition, #(T~N: Visitor<'a, E, S>,)*> Visitor<'a, E, S> for (#(T~N,)*) {
                fn new(
                    executable_document: &'a E,
                    schema_definition: &'a S,
                    cache: &'a Cache<'a, E, S>,
                ) -> Self {
                    (
                        #(T~N::new(
                            executable_document,
                            schema_definition,
                            cache,
                        ),)*
                    )
                }

                fn visit_operation_definition(&mut self, operation_definition: &'a E::OperationDefinition) {
                    #(self.N.visit_operation_definition(operation_definition);)*
                }

                fn visit_selection_set(
                    &mut self,
                    selection_set: &'a E::SelectionSet,
                    r#type: TypeDefinitionReference<'a, S::TypeDefinition>,
                ) {
                    #(self.N.visit_selection_set(selection_set, r#type);)*
                }

                fn visit_field(
                    &mut self,
                    field: &'a E::Field,
                    field_definition: &'a S::FieldDefinition,
                    path: &Path<'a, E>,
                ) {
                    #(self.N.visit_field(field, field_definition, path);)*
                }

                fn visit_const_directive(
                    &mut self,
                    directive: &'a E::Directive<true>,
                    location: DirectiveLocation,
                ) {
                    #(self.N.visit_const_directive(directive, location);)*
                }

                fn visit_variable_directive(
                    &mut self,
                    directive: &'a E::Directive<false>,
                    location: DirectiveLocation,
                ) {
                    #(self.N.visit_variable_directive(directive, location);)*
                }

                fn visit_const_directives(
                    &mut self,
                    directives: &'a E::Directives<true>,
                    location: DirectiveLocation,
                ) {
                    #(self.N.visit_const_directives(directives, location);)*
                }

                fn visit_variable_directives(
                    &mut self,
                    directives: &'a E::Directives<false>,
                    location: DirectiveLocation,
                ) {
                    #(self.N.visit_variable_directives(directives, location);)*
                }

                fn visit_fragment_definition(&mut self, fragment_definition: &'a E::FragmentDefinition) {
                    #(self.N.visit_fragment_definition(fragment_definition);)*
                }

                fn visit_inline_fragment(
                    &mut self,
                    inline_fragment: &'a E::InlineFragment,
                    scoped_type: TypeDefinitionReference<'a, S::TypeDefinition>,
                ) {
                    #(self.N.visit_inline_fragment(inline_fragment, scoped_type);)*
                }

                fn visit_fragment_spread(
                    &mut self,
                    fragment_spread: &'a E::FragmentSpread,
                    scoped_type: TypeDefinitionReference<'a, S::TypeDefinition>,
                    path: &Path<'a, E>,
                ) {
                    #(self.N.visit_fragment_spread(fragment_spread, scoped_type, path);)*
                }

                fn visit_const_argument(
                    &mut self,
                    argument: &'a E::Argument<true>,
                    input_value_definition: &'a S::InputValueDefinition,
                ) {
                    #(self.N.visit_const_argument(argument, input_value_definition);)*
                }

                fn visit_variable_argument(
                    &mut self,
                    argument: &'a E::Argument<false>,
                    input_value_definition: &'a S::InputValueDefinition,
                    path: &Path<'a, E>,
                ) {
                    #(self.N.visit_variable_argument(argument, input_value_definition, path);)*
                }

                fn visit_variable_definition(&mut self, variable_definition: &'a E::VariableDefinition) {
                    #(self.N.visit_variable_definition(variable_definition);)*
                }

                fn visit_variable_definitions(&mut self, variable_definitions: &'a E::VariableDefinitions) {
                    #(self.N.visit_variable_definitions(variable_definitions);)*
                }
            }
        });
    }
}

seq_macro::seq!(N in 2..=10 {
    impl_visitor!(N);
});

impl_visitor!(26);
