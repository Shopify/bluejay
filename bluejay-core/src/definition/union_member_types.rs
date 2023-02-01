use crate::definition::UnionMemberType;
use crate::AsIter;

pub trait UnionMemberTypes: AsIter<Item = Self::UnionMemberType> {
    type UnionMemberType: UnionMemberType;
}
