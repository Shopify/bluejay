use crate::{Cache, Directives, FieldsDefinition, UnionMemberTypes, Warden};
use bluejay_core::definition::{self, HasDirectives, SchemaDefinition};
use once_cell::unsync::OnceCell;

pub struct UnionTypeDefinition<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::UnionTypeDefinition,
    cache: &'a Cache<'a, S, W>,
    union_member_types: OnceCell<UnionMemberTypes<'a, S, W>>,
    fields_definition: OnceCell<FieldsDefinition<'a, S, W>>,
    directives: Option<Directives<'a, S, W>>,
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> UnionTypeDefinition<'a, S, W> {
    pub(crate) fn new(inner: &'a S::UnionTypeDefinition, cache: &'a Cache<'a, S, W>) -> Self {
        Self {
            inner,
            cache,
            union_member_types: OnceCell::new(),
            fields_definition: OnceCell::new(),
            directives: HasDirectives::directives(inner).map(|d| Directives::new(d, cache)),
        }
    }

    pub fn inner(&self) -> &'a S::UnionTypeDefinition {
        self.inner
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::UnionTypeDefinition
    for UnionTypeDefinition<'a, S, W>
{
    type UnionMemberTypes = UnionMemberTypes<'a, S, W>;
    type FieldsDefinition = FieldsDefinition<'a, S, W>;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn name(&self) -> &str {
        self.inner.name()
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

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> HasDirectives
    for UnionTypeDefinition<'a, S, W>
{
    type Directives = Directives<'a, S, W>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}
