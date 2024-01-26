use crate::definition::{InterfaceTypeDefinition, SchemaDefinition};

pub trait InterfaceImplementation {
    type InterfaceTypeDefinition: InterfaceTypeDefinition;

    fn interface<'a, S: SchemaDefinition<InterfaceTypeDefinition = Self::InterfaceTypeDefinition>>(
        &'a self,
        schema_definition: &'a S,
    ) -> &'a Self::InterfaceTypeDefinition;

    fn name(&self) -> &str;
}
