use crate::{DirectiveDefinition, TypeDefinition, Warden};
use bluejay_core::definition::{prelude::*, SchemaDefinition, TypeDefinitionReference};
use elsa::FrozenMap;

pub struct Cache<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    warden: W,
    type_definitions: FrozenMap<&'a str, Box<TypeDefinition<'a, S, W>>>,
    directive_definitions: FrozenMap<&'a str, Box<DirectiveDefinition<'a, S, W>>>,
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> Cache<'a, S, W> {
    pub fn new(warden: W) -> Self {
        Self {
            warden,
            type_definitions: FrozenMap::new(),
            directive_definitions: FrozenMap::new(),
        }
    }

    pub(crate) fn warden(&self) -> &W {
        &self.warden
    }

    pub(crate) fn get_or_create_type_definition(
        &'a self,
        type_definition: TypeDefinitionReference<'a, S::TypeDefinition>,
    ) -> Option<&'a TypeDefinition<'a, S, W>> {
        self.type_definitions
            .get(type_definition.name())
            .or_else(|| {
                TypeDefinition::new(type_definition, self).map(|td| {
                    self.type_definitions
                        .insert(type_definition.name(), Box::new(td))
                })
            })
    }

    pub(crate) fn get_type_definition(
        &'a self,
        name: &str,
    ) -> Option<&'a TypeDefinition<'a, S, W>> {
        self.type_definitions.get(name)
    }

    pub(crate) fn get_or_create_directive_definition(
        &'a self,
        directive_definition: &'a S::DirectiveDefinition,
    ) -> &'a DirectiveDefinition<'a, S, W> {
        self.directive_definitions
            .get(directive_definition.name())
            .unwrap_or_else(|| {
                self.directive_definitions.insert(
                    directive_definition.name(),
                    Box::new(DirectiveDefinition::new(directive_definition, self)),
                )
            })
    }

    pub(crate) fn get_directive_definition(
        &'a self,
        name: &str,
    ) -> Option<&'a DirectiveDefinition<'a, S, W>> {
        self.directive_definitions.get(name)
    }
}
