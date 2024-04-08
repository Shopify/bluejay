use crate::{DirectiveDefinition, TypeDefinition, Warden};
use bluejay_core::definition::{prelude::*, SchemaDefinition, TypeDefinitionReference};
use elsa::FrozenMap;

pub struct Cache<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    warden: W,
    inner_schema_definition: &'a S,
    type_definitions: FrozenMap<&'a str, Box<TypeDefinition<'a, S, W>>>,
    directive_definitions: FrozenMap<&'a str, Box<DirectiveDefinition<'a, S, W>>>,
}

impl<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> Cache<'a, S, W> {
    pub fn new(warden: W, inner_schema_definition: &'a S) -> Self {
        Self {
            warden,
            inner_schema_definition,
            type_definitions: FrozenMap::new(),
            directive_definitions: FrozenMap::new(),
        }
    }

    pub fn warden(&self) -> &W {
        &self.warden
    }

    pub(crate) fn inner_schema_definition(&self) -> &'a S {
        self.inner_schema_definition
    }

    pub(crate) fn get_or_create_type_definition(
        &'a self,
        type_definition: TypeDefinitionReference<'a, S::TypeDefinition>,
    ) -> Option<&'a TypeDefinition<'a, S, W>> {
        match self.type_definitions.get(type_definition.name()) {
            Some(existing_type_definition) => self
                .type_definitions_equal(existing_type_definition.inner(), type_definition)
                .then_some(existing_type_definition),
            None => TypeDefinition::new(type_definition, self).map(|td| {
                self.type_definitions
                    .insert(type_definition.name(), Box::new(td))
            }),
        }
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
    ) -> Option<&'a DirectiveDefinition<'a, S, W>> {
        self.directive_definitions
            .get(directive_definition.name())
            .or_else(|| {
                DirectiveDefinition::new(directive_definition, self).map(|dd| {
                    self.directive_definitions
                        .insert(directive_definition.name(), Box::new(dd))
                })
            })
    }

    pub(crate) fn get_directive_definition(
        &'a self,
        name: &str,
    ) -> Option<&'a DirectiveDefinition<'a, S, W>> {
        self.directive_definitions.get(name)
    }

    fn type_definitions_equal(
        &self,
        left: TypeDefinitionReference<'a, S::TypeDefinition>,
        right: TypeDefinitionReference<'a, S::TypeDefinition>,
    ) -> bool {
        match (left, right) {
            (
                TypeDefinitionReference::BuiltinScalar(left),
                TypeDefinitionReference::BuiltinScalar(right),
            ) => left == right,
            (
                TypeDefinitionReference::CustomScalar(left),
                TypeDefinitionReference::CustomScalar(right),
            ) => self.warden.scalar_type_definitions_equal(left, right),
            (TypeDefinitionReference::Enum(left), TypeDefinitionReference::Enum(right)) => {
                self.warden.enum_type_definitions_equal(left, right)
            }
            (
                TypeDefinitionReference::InputObject(left),
                TypeDefinitionReference::InputObject(right),
            ) => self.warden.input_object_type_definitions_equal(left, right),
            (
                TypeDefinitionReference::Interface(left),
                TypeDefinitionReference::Interface(right),
            ) => self.warden.interface_type_definitions_equal(left, right),
            (TypeDefinitionReference::Object(left), TypeDefinitionReference::Object(right)) => {
                self.warden.object_type_definitions_equal(left, right)
            }
            (TypeDefinitionReference::Union(left), TypeDefinitionReference::Union(right)) => {
                self.warden.union_type_definitions_equal(left, right)
            }
            _ => false,
        }
    }
}
