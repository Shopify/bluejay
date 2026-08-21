use bluejay_core::definition::SchemaDefinition;

pub trait Visitor<'a, S: SchemaDefinition> {
    fn new(schema_definition: &'a S) -> Self;

    fn visit_input_object_type_definition(
        &mut self,
        _input_object_type_definition: &'a S::InputObjectTypeDefinition,
    ) {
    }

    fn visit_enum_type_definition(&mut self, _enum_type_definition: &'a S::EnumTypeDefinition) {}
}

macro_rules! impl_visitor {
    ($n:literal) => {
        seq_macro::seq!(N in 0..$n {
            #[warn(clippy::missing_trait_methods)]
            impl<'a, S: SchemaDefinition, #(T~N: Visitor<'a, S>,)*> Visitor<'a, S> for (#(T~N,)*) {
                fn new(schema_definition: &'a S) -> Self {
                    (#(T~N::new(schema_definition),)*)
                }

                fn visit_input_object_type_definition(
                    &mut self,
                    input_object_type_definition: &'a S::InputObjectTypeDefinition,
                ) {
                    #(self.N.visit_input_object_type_definition(input_object_type_definition);)*
                }

                fn visit_enum_type_definition(
                    &mut self,
                    enum_type_definition: &'a S::EnumTypeDefinition,
                ) {
                    #(self.N.visit_enum_type_definition(enum_type_definition);)*
                }
            }
        });
    }
}

seq_macro::seq!(N in 2..=10 {
    impl_visitor!(N);
});
