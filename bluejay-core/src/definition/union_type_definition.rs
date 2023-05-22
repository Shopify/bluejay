use crate::definition::{FieldsDefinition, UnionMemberTypes};
use crate::ConstDirectives;

pub trait UnionTypeDefinition {
    type UnionMemberTypes: UnionMemberTypes;
    type Directives: ConstDirectives;
    type FieldsDefinition: FieldsDefinition;

    fn description(&self) -> Option<&str>;
    fn name(&self) -> &str;
    fn directives(&self) -> Option<&Self::Directives>;
    fn union_member_types(&self) -> &Self::UnionMemberTypes;
    /// Should only contain the builtin `__typename` field definition
    fn fields_definition(&self) -> &Self::FieldsDefinition;
}
