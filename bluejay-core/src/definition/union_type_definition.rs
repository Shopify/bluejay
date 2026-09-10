use crate::definition::{HasDirectives, SchemaDefinition};

pub trait UnionTypeDefinition:
    HasDirectives<Directives = <Self::SchemaDefinition as SchemaDefinition>::Directives>
{
    type SchemaDefinition: SchemaDefinition;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn union_member_types(&self)
        -> &<Self::SchemaDefinition as SchemaDefinition>::UnionMemberTypes;
    /// Should only contain the builtin `__typename` field definition
    fn fields_definition(&self) -> &<Self::SchemaDefinition as SchemaDefinition>::FieldsDefinition;
}
