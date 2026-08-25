use crate::definition::{HasDirectives, SchemaDefinition};

pub trait EnumTypeDefinition:
    HasDirectives<Directives = <Self::SchemaDefinition as SchemaDefinition>::Directives>
{
    type SchemaDefinition: SchemaDefinition;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn enum_value_definitions(
        &self,
    ) -> &<Self::SchemaDefinition as SchemaDefinition>::EnumValueDefinitions;
    fn is_builtin(&self) -> bool;
}
