use crate::{Cache, EnumValueDefinitions, Warden};
use bluejay_core::definition::{self, SchemaDefinition};
use once_cell::unsync::OnceCell;

pub struct EnumTypeDefinition<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::EnumTypeDefinition,
    cache: &'a Cache<'a, S, W>,
    enum_value_definitions: OnceCell<EnumValueDefinitions<'a, S, W>>,
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> EnumTypeDefinition<'a, S, W> {
    pub(crate) fn new(inner: &'a S::EnumTypeDefinition, cache: &'a Cache<'a, S, W>) -> Self {
        Self {
            inner,
            cache,
            enum_value_definitions: OnceCell::new(),
        }
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::EnumTypeDefinition
    for EnumTypeDefinition<'a, S, W>
{
    type Directives = <S::EnumTypeDefinition as definition::EnumTypeDefinition>::Directives;
    type EnumValueDefinitions = EnumValueDefinitions<'a, S, W>;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.inner.directives()
    }

    fn enum_value_definitions(&self) -> &Self::EnumValueDefinitions {
        self.enum_value_definitions.get_or_init(|| {
            EnumValueDefinitions::new(self.inner.enum_value_definitions(), self.cache)
        })
    }

    fn is_builtin(&self) -> bool {
        self.inner.is_builtin()
    }
}
