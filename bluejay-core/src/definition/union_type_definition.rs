use crate::definition::UnionMemberTypes;
use crate::ConstDirectives;

pub trait UnionTypeDefinition {
    type UnionMemberTypes: UnionMemberTypes;
    type Directives: ConstDirectives;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn directives(&self) -> Option<&Self::Directives>;
    fn union_member_types(&self) -> &Self::UnionMemberTypes;
}
