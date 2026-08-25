use crate::definition::{HasDirectives, SchemaDefinition};

pub trait FieldDefinition:
    HasDirectives<Directives = <Self::SchemaDefinition as SchemaDefinition>::Directives>
{
    type SchemaDefinition: SchemaDefinition;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn arguments_definition(
        &self,
    ) -> Option<&<Self::SchemaDefinition as SchemaDefinition>::ArgumentsDefinition>;
    fn r#type(&self) -> &<Self::SchemaDefinition as SchemaDefinition>::OutputType;
    fn is_builtin(&self) -> bool;
}
