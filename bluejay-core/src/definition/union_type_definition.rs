use crate::definition::{FieldsDefinition, HasDirectives, UnionMemberTypes};

pub trait UnionTypeDefinition: HasDirectives {
    type UnionMemberTypes: UnionMemberTypes;
    type FieldsDefinition: FieldsDefinition;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn union_member_types(&self) -> &Self::UnionMemberTypes;
    /// Should only contain the builtin `__typename` field definition
    fn fields_definition(&self) -> &Self::FieldsDefinition;
}
