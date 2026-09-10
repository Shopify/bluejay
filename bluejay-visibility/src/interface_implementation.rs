use crate::{Cache, InterfaceTypeDefinition, Warden};
use bluejay_core::definition::{self, prelude::*, SchemaDefinition, TypeDefinitionReference};

pub struct InterfaceImplementation<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S::InterfaceImplementation,
    interface: &'a InterfaceTypeDefinition<'a, S, W>,
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> InterfaceImplementation<'a, S, W> {
    pub(crate) fn new(
        inner: &'a S::InterfaceImplementation,
        cache: &'a Cache<'a, S, W>,
    ) -> Option<Self> {
        cache
            .warden()
            .is_interface_implementation_visible(inner)
            .then(|| {
                cache
                    .get_or_create_type_definition(TypeDefinitionReference::Interface(
                        definition::InterfaceImplementation::interface(
                            inner,
                            cache.inner_schema_definition(),
                        ),
                    ))
                    .map(|td| Self {
                        inner,
                        interface: td.as_interface().unwrap(),
                    })
            })
            .flatten()
    }

    pub fn inner(&self) -> &'a S::InterfaceImplementation {
        self.inner
    }

    pub(crate) fn interface(&self) -> &'a InterfaceTypeDefinition<'a, S, W> {
        self.interface
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>>
    definition::InterfaceImplementation for InterfaceImplementation<'a, S, W>
{
    type SchemaDefinition = crate::SchemaDefinition<'a, S, W>;

    fn interface<'b>(
        &'b self,
        _: &'b Self::SchemaDefinition,
    ) -> &'b InterfaceTypeDefinition<'a, S, W> {
        self.interface
    }

    fn name(&self) -> &str {
        self.interface.name()
    }
}
