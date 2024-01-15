use crate::definition::UnionMemberType;
use crate::AsIter;

pub trait UnionMemberTypes: AsIter<Item = Self::UnionMemberType> {
    type UnionMemberType: UnionMemberType;

    fn contains_type(&self, name: &str) -> bool {
        self.iter().any(|t| t.name() == name)
    }

    fn get(&self, name: &str) -> Option<&Self::UnionMemberType> {
        self.iter().find(|t| t.name() == name)
    }
}
