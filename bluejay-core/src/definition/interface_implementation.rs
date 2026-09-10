use crate::definition::SchemaDefinition;

pub trait InterfaceImplementation {
    type SchemaDefinition: SchemaDefinition;

    fn interface<'a>(
        &'a self,
        schema_definition: &'a Self::SchemaDefinition,
    ) -> &'a <Self::SchemaDefinition as SchemaDefinition>::InterfaceTypeDefinition;

    fn name(&self) -> &str;
}
