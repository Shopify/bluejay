use crate::executable::{operation::VariableValues, Cache};
use bluejay_core::definition::{SchemaDefinition, TypeDefinitionReference};
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
    /// # Variables
    /// - `field` is the field being visited
    /// - `field_definition` is the definition of the field
    /// - `scoped_type` is the type that the field definition is defined within
    /// - `included` is true when the field is known to be included in the response
    ///   (based on the usage of `@include` and `@skip` directives and the variable values)
    #[allow(unused_variables)]
    fn visit_field(
        &mut self,
        field: &'a E::Field,
        field_definition: &'a S::FieldDefinition,
        scoped_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        included: bool,
    ) {
    }

    /// Called after the field and all of its children have been visited.
    /// See `visit_field` for more information about the variables.
    #[allow(unused_variables)]
    fn leave_field(
        &mut self,
        field: &'a <E as ExecutableDocument>::Field,
        field_definition: &'a S::FieldDefinition,
        scoped_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        included: bool,
    ) {
    }

    /// Visits the variable definition.
    /// # Variables
    /// - `variable_definition` is the variable definition being visited
    #[allow(unused_variables)]
    fn visit_variable_definition(&mut self, variable_definition: &'a E::VariableDefinition) {}
}

macro_rules! impl_visitor {
    ($n:literal) => {
        seq_macro::seq!(N in 0..$n {
            #[warn(clippy::missing_trait_methods)]
            impl<'a, E: ExecutableDocument, S: SchemaDefinition, V: VariableValues, #(T~N: Visitor<'a, E, S, V>,)*> Visitor<'a, E, S, V> for (#(T~N,)*) {
                fn new(
                    operation_definition: &'a E::OperationDefinition,
                    schema_definition: &'a S,
                    variable_values: &'a V,
                    cache: &'a Cache<'a, E, S>,
                ) -> Self {
                    (
                        #(T~N::new(
                            operation_definition,
                            schema_definition,
                            variable_values,
                            cache,
                        ),)*
                    )
                }

                fn visit_field(
                    &mut self,
                    field: &'a E::Field,
                    field_definition: &'a S::FieldDefinition,
                    owner_type: TypeDefinitionReference<'a, S::TypeDefinition>,
                    included: bool,
                ) {
                    #(self.N.visit_field(field, field_definition, owner_type, included);)*
                }

                fn leave_field(
                    &mut self,
                    field: &'a <E as ExecutableDocument>::Field,
                    field_definition: &'a S::FieldDefinition,
                    owner_type: TypeDefinitionReference<'a, S::TypeDefinition>,
                    included: bool,
                ) {
                    #(self.N.leave_field(field, field_definition, owner_type, included);)*
                }

                fn visit_variable_definition(&mut self, variable_definition: &'a E::VariableDefinition) {
                    #(self.N.visit_variable_definition(variable_definition);)*
                }
            }
        });
    }
}

seq_macro::seq!(N in 2..=10 {
    impl_visitor!(N);
});
