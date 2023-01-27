use crate::definition::UnionMemberTypes;

pub trait UnionTypeDefinition {
    type UnionMemberTypes: UnionMemberTypes;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn union_member_types(&self) -> &Self::UnionMemberTypes;
}
