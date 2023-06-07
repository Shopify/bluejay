use crate::definition::{ObjectTypeDefinition, UnionMemberType};
use crate::AsIter;

pub trait UnionMemberTypes: AsIter<Item = Self::UnionMemberType> {
    type UnionMemberType: UnionMemberType;

    fn contains_type(&self, name: &str) -> bool {
        self.iter().any(|t| t.member_type().name() == name)
    }

    fn get(&self, name: &str) -> Option<&Self::UnionMemberType> {
        self.iter().find(|t| t.member_type().name() == name)
    }
}
