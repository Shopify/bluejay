use crate::{Cache, ObjectTypeDefinition, Warden};
use bluejay_core::definition::{self, SchemaDefinition, TypeDefinitionReference};
use once_cell::unsync::OnceCell;

pub struct UnionMemberType<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::UnionMemberType,
    cache: &'a Cache<'a, S, W>,
    member_type: OnceCell<&'a ObjectTypeDefinition<'a, S, W>>,
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> UnionMemberType<'a, S, W> {
    pub(crate) fn new(inner: &'a S::UnionMemberType, cache: &'a Cache<'a, S, W>) -> Option<Self> {
        cache
            .warden()
            .is_union_member_type_visible(inner)
            .then_some(Self {
                inner,
                cache,
                member_type: OnceCell::new(),
            })
    }

    pub fn inner(&self) -> &'a S::UnionMemberType {
        self.inner
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::UnionMemberType
    for UnionMemberType<'a, S, W>
{
    type ObjectTypeDefinition = ObjectTypeDefinition<'a, S, W>;

    fn member_type(&self) -> &Self::ObjectTypeDefinition {
        self.member_type.get_or_init(|| {
            self.cache
                .get_or_create_type_definition(TypeDefinitionReference::Object(
                    self.inner.member_type(),
                ))
                .as_object()
                .unwrap()
        })
    }
}
