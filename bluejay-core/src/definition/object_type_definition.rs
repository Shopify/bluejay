use crate::definition::{HasDirectives, SchemaDefinition};

pub trait ObjectTypeDefinition:
    HasDirectives<Directives = <Self::SchemaDefinition as SchemaDefinition>::Directives>
{
    type SchemaDefinition: SchemaDefinition;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn interface_implementations(
        &self,
    ) -> Option<&<Self::SchemaDefinition as SchemaDefinition>::InterfaceImplementations>;
    fn fields_definition(&self) -> &<Self::SchemaDefinition as SchemaDefinition>::FieldsDefinition;
    fn is_builtin(&self) -> bool;
}
