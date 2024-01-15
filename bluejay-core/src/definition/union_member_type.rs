use crate::definition::{ObjectTypeDefinition, SchemaDefinition};

pub trait UnionMemberType {
    type ObjectTypeDefinition: ObjectTypeDefinition;

    fn member_type<'a, S: SchemaDefinition<ObjectTypeDefinition = Self::ObjectTypeDefinition>>(
        &'a self,
        schema_definition: &'a S,
    ) -> &'a Self::ObjectTypeDefinition;
    fn name(&self) -> &str;
}
