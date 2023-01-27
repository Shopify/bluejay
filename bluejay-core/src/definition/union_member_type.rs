use crate::definition::ObjectTypeDefinition;

pub trait UnionMemberType {
    type ObjectTypeDefinition: ObjectTypeDefinition;

    fn member_type(&self) -> &Self::ObjectTypeDefinition;
}
