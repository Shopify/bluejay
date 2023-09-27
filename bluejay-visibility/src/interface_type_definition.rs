use crate::{Cache, Directives, FieldsDefinition, InterfaceImplementations, Warden};
use bluejay_core::definition::{self, HasDirectives, SchemaDefinition};
use once_cell::unsync::OnceCell;

pub struct InterfaceTypeDefinition<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::InterfaceTypeDefinition,
    cache: &'a Cache<'a, S, W>,
    fields_definition: OnceCell<FieldsDefinition<'a, S, W>>,
    interface_implementations: OnceCell<Option<InterfaceImplementations<'a, S, W>>>,
    directives: Option<Directives<'a, S, W>>,
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>>
    InterfaceTypeDefinition<'a, S, W>
{
    pub(crate) fn new(inner: &'a S::InterfaceTypeDefinition, cache: &'a Cache<'a, S, W>) -> Self {
        Self {
            inner,
            cache,
            fields_definition: OnceCell::new(),
            interface_implementations: OnceCell::new(),
            directives: inner.directives().map(|d| Directives::new(d, cache)),
        }
    }

    pub fn inner(&self) -> &'a S::InterfaceTypeDefinition {
        self.inner
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>>
    definition::InterfaceTypeDefinition for InterfaceTypeDefinition<'a, S, W>
{
    type FieldsDefinition = FieldsDefinition<'a, S, W>;
    type InterfaceImplementations = InterfaceImplementations<'a, S, W>;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn fields_definition(&self) -> &Self::FieldsDefinition {
        self.fields_definition
            .get_or_init(|| FieldsDefinition::new(self.inner.fields_definition(), self.cache))
    }

    fn interface_implementations(&self) -> Option<&Self::InterfaceImplementations> {
        self.interface_implementations
            .get_or_init(|| {
                self.inner
                    .interface_implementations()
                    .map(|ii| InterfaceImplementations::new(ii, self.cache))
            })
            .as_ref()
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> HasDirectives
    for InterfaceTypeDefinition<'a, S, W>
{
    type Directives = Directives<'a, S, W>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}
