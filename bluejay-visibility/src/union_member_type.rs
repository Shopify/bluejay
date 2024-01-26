use crate::{Cache, ObjectTypeDefinition, Warden};
use bluejay_core::definition::{self, SchemaDefinition, TypeDefinitionReference};

pub struct UnionMemberType<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::UnionMemberType,
    member_type: &'a ObjectTypeDefinition<'a, S, W>,
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> UnionMemberType<'a, S, W> {
    pub(crate) fn new(inner: &'a S::UnionMemberType, cache: &'a Cache<'a, S, W>) -> Option<Self> {
        cache
            .warden()
            .is_union_member_type_visible(inner)
            .then(|| {
                cache
                    .get_or_create_type_definition(TypeDefinitionReference::Object(
                        definition::UnionMemberType::member_type(
                            inner,
                            cache.inner_schema_definition(),
                        ),
                    ))
                    .map(|td| Self {
                        inner,
                        member_type: td.as_object().unwrap(),
                    })
            })
            .flatten()
    }

    pub fn inner(&self) -> &'a S::UnionMemberType {
        self.inner
    }

    pub(crate) fn member_type(&self) -> &'a ObjectTypeDefinition<'a, S, W> {
        self.member_type
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::UnionMemberType
    for UnionMemberType<'a, S, W>
{
    type ObjectTypeDefinition = ObjectTypeDefinition<'a, S, W>;

    fn member_type<'b, S2: SchemaDefinition<ObjectTypeDefinition = Self::ObjectTypeDefinition>>(
        &'b self,
        _: &'b S2,
    ) -> &'b Self::ObjectTypeDefinition {
        self.member_type
    }

    fn name(&self) -> &str {
        self.inner.name()
    }
}
