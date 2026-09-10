use crate::definition::{HasDirectives, SchemaDefinition};

pub trait InputObjectTypeDefinition:
    HasDirectives<Directives = <Self::SchemaDefinition as SchemaDefinition>::Directives>
{
    type SchemaDefinition: SchemaDefinition;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn input_field_definitions(
        &self,
    ) -> &<Self::SchemaDefinition as SchemaDefinition>::InputFieldsDefinition;
}
