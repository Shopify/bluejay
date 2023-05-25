use bluejay_core::definition::SchemaDefinition;

pub trait Visitor<'a, S: SchemaDefinition> {
    fn visit_input_object_type_definition(
        &mut self,
        _input_object_type_definition: &'a S::InputObjectTypeDefinition,
    ) {
    }

    fn visit_enum_type_definition(&mut self, _enum_type_definition: &'a S::EnumTypeDefinition) {}
}
