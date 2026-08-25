use crate::definition::SchemaDefinition;

pub trait UnionMemberType {
    type SchemaDefinition: SchemaDefinition;

    fn member_type<'a>(
        &'a self,
        schema_definition: &'a Self::SchemaDefinition,
    ) -> &'a <Self::SchemaDefinition as SchemaDefinition>::ObjectTypeDefinition;
    fn name(&self) -> &str;
}
