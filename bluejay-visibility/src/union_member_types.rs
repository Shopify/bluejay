use crate::{Cache, UnionMemberType, Warden};
use bluejay_core::definition::{self, SchemaDefinition};
use bluejay_core::AsIter;
use once_cell::unsync::OnceCell;

pub struct UnionMemberTypes<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::UnionMemberTypes,
    cache: &'a Cache<'a, S, W>,
    member_types: OnceCell<Vec<UnionMemberType<'a, S, W>>>,
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> UnionMemberTypes<'a, S, W> {
    pub(crate) fn new(inner: &'a S::UnionMemberTypes, cache: &'a Cache<'a, S, W>) -> Self {
        Self {
            inner,
            cache,
            member_types: OnceCell::new(),
        }
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> AsIter
    for UnionMemberTypes<'a, S, W>
{
    type Item = UnionMemberType<'a, S, W>;
    type Iterator<'b>
        = std::slice::Iter<'b, Self::Item>
    where
        'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.member_types
            .get_or_init(|| {
                self.inner
                    .iter()
                    .filter_map(|mt| UnionMemberType::new(mt, self.cache))
                    .collect()
            })
            .iter()
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::UnionMemberTypes
    for UnionMemberTypes<'a, S, W>
{
    type UnionMemberType = UnionMemberType<'a, S, W>;
}
