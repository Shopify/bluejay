use crate::{Cache, FieldsDefinition, UnionMemberTypes, Warden};
use bluejay_core::definition::{self, SchemaDefinition};
use once_cell::unsync::OnceCell;

pub struct UnionTypeDefinition<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::UnionTypeDefinition,
    cache: &'a Cache<'a, S, W>,
    union_member_types: OnceCell<UnionMemberTypes<'a, S, W>>,
    fields_definition: OnceCell<FieldsDefinition<'a, S, W>>,
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> UnionTypeDefinition<'a, S, W> {
    pub(crate) fn new(inner: &'a S::UnionTypeDefinition, cache: &'a Cache<'a, S, W>) -> Self {
        Self {
            inner,
            cache,
            union_member_types: OnceCell::new(),
            fields_definition: OnceCell::new(),
        }
    }

    pub fn inner(&self) -> &'a S::UnionTypeDefinition {
        self.inner
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::UnionTypeDefinition
    for UnionTypeDefinition<'a, S, W>
{
    type Directives = <S::UnionTypeDefinition as definition::UnionTypeDefinition>::Directives;
    type UnionMemberTypes = UnionMemberTypes<'a, S, W>;
    type FieldsDefinition = FieldsDefinition<'a, S, W>;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.inner.directives()
    }

    fn union_member_types(&self) -> &Self::UnionMemberTypes {
        self.union_member_types
            .get_or_init(|| UnionMemberTypes::new(self.inner.union_member_types(), self.cache))
    }

    fn fields_definition(&self) -> &Self::FieldsDefinition {
        self.fields_definition
            .get_or_init(|| FieldsDefinition::new(self.inner.fields_definition(), self.cache))
    }
}
