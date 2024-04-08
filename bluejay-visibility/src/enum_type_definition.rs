use crate::{Cache, Directives, EnumValueDefinitions, Warden};
use bluejay_core::definition::{self, HasDirectives, SchemaDefinition};
use once_cell::unsync::OnceCell;

pub struct EnumTypeDefinition<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::EnumTypeDefinition,
    cache: &'a Cache<'a, S, W>,
    enum_value_definitions: OnceCell<EnumValueDefinitions<'a, S, W>>,
    directives: Option<Directives<'a, S, W>>,
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> EnumTypeDefinition<'a, S, W> {
    pub(crate) fn new(inner: &'a S::EnumTypeDefinition, cache: &'a Cache<'a, S, W>) -> Self {
        Self {
            inner,
            cache,
            enum_value_definitions: OnceCell::new(),
            directives: inner.directives().map(|d| Directives::new(d, cache)),
        }
    }

    pub(crate) fn inner(&self) -> &'a S::EnumTypeDefinition {
        self.inner
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::EnumTypeDefinition
    for EnumTypeDefinition<'a, S, W>
{
    type EnumValueDefinitions = EnumValueDefinitions<'a, S, W>;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn name(&self) -> &str {
        self.inner.name()
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

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> HasDirectives
    for EnumTypeDefinition<'a, S, W>
{
    type Directives = Directives<'a, S, W>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}
