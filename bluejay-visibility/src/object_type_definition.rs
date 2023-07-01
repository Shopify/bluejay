use crate::{Cache, FieldsDefinition, InterfaceImplementations, Warden};
use bluejay_core::definition::{self, SchemaDefinition};
use once_cell::unsync::OnceCell;

pub struct ObjectTypeDefinition<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::ObjectTypeDefinition,
    cache: &'a Cache<'a, S, W>,
    fields_definition: OnceCell<FieldsDefinition<'a, S, W>>,
    interface_implementations: OnceCell<Option<InterfaceImplementations<'a, S, W>>>,
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> ObjectTypeDefinition<'a, S, W> {
    pub(crate) fn new(inner: &'a S::ObjectTypeDefinition, cache: &'a Cache<'a, S, W>) -> Self {
        Self {
            inner,
            cache,
            fields_definition: OnceCell::new(),
            interface_implementations: OnceCell::new(),
        }
    }

    pub fn inner(&self) -> &'a S::ObjectTypeDefinition {
        self.inner
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::ObjectTypeDefinition
    for ObjectTypeDefinition<'a, S, W>
{
    type Directives = <S::ObjectTypeDefinition as definition::ObjectTypeDefinition>::Directives;
    type FieldsDefinition = FieldsDefinition<'a, S, W>;
    type InterfaceImplementations = InterfaceImplementations<'a, S, W>;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.inner.directives()
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

    fn is_builtin(&self) -> bool {
        self.inner.is_builtin()
    }
}
