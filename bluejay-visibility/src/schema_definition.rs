use crate::{
    ArgumentsDefinition, BaseInputType, BaseOutputType, Cache, DirectiveDefinition, Directives,
    EnumTypeDefinition, EnumValueDefinition, EnumValueDefinitions, FieldDefinition,
    FieldsDefinition, InputFieldsDefinition, InputObjectTypeDefinition, InputType,
    InputValueDefinition, InterfaceImplementation, InterfaceImplementations,
    InterfaceTypeDefinition, ObjectTypeDefinition, OutputType, ScalarTypeDefinition,
    TypeDefinition, UnionMemberType, UnionMemberTypes, UnionTypeDefinition, Warden,
};
use bluejay_core::definition::{self, prelude::*};
use bluejay_core::AsIter;
use elsa::FrozenMap;
use once_cell::unsync::OnceCell;

pub struct SchemaDefinition<'a, S: definition::SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    inner: &'a S,
    cache: &'a Cache<'a, S, W>,
    query: ObjectTypeDefinition<'a, S, W>,
    mutation: Option<ObjectTypeDefinition<'a, S, W>>,
    subscription: Option<ObjectTypeDefinition<'a, S, W>>,
    interface_implementors: FrozenMap<&'a str, Vec<&'a ObjectTypeDefinition<'a, S, W>>>,
    type_definitions: OnceCell<Vec<&'a TypeDefinition<'a, S, W>>>,
    directive_definitions: OnceCell<Vec<&'a DirectiveDefinition<'a, S, W>>>,
    schema_directives: Option<Directives<'a, S, W>>,
}

impl<'a, S: definition::SchemaDefinition, W: Warden<SchemaDefinition = S>>
    SchemaDefinition<'a, S, W>
{
    pub fn new(inner: &'a S, cache: &'a Cache<'a, S, W>) -> Self {
        Self {
            inner,
            cache,
            query: ObjectTypeDefinition::new(inner.query(), cache),
            mutation: inner
                .mutation()
                .map(|mutation| ObjectTypeDefinition::new(mutation, cache)),
            subscription: inner
                .subscription()
                .map(|subscription| ObjectTypeDefinition::new(subscription, cache)),
            interface_implementors: FrozenMap::new(),
            type_definitions: OnceCell::new(),
            directive_definitions: OnceCell::new(),
            schema_directives: definition::SchemaDefinition::schema_directives(inner)
                .map(|d| Directives::new(d, cache)),
        }
    }

    pub fn inner(&self) -> &'a S {
        self.inner
    }
}

impl<'a, S: definition::SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>>
    definition::SchemaDefinition for SchemaDefinition<'a, S, W>
{
    type Directive = S::Directive;
    type Directives = Directives<'a, S, W>;
    type InputValueDefinition = InputValueDefinition<'a, S, W>;
    type InputFieldsDefinition = InputFieldsDefinition<'a, S, W>;
    type ArgumentsDefinition = ArgumentsDefinition<'a, S, W>;
    type EnumValueDefinition = EnumValueDefinition<'a, S, W>;
    type EnumValueDefinitions = EnumValueDefinitions<'a, S, W>;
    type FieldDefinition = FieldDefinition<'a, S, W>;
    type FieldsDefinition = FieldsDefinition<'a, S, W>;
    type InterfaceImplementation = InterfaceImplementation<'a, S, W>;
    type InterfaceImplementations = InterfaceImplementations<'a, S, W>;
    type UnionMemberType = UnionMemberType<'a, S, W>;
    type UnionMemberTypes = UnionMemberTypes<'a, S, W>;
    type BaseInputType = BaseInputType<'a, S, W>;
    type InputType = InputType<'a, S, W>;
    type BaseOutputType = BaseOutputType<'a, S, W>;
    type OutputType = OutputType<'a, S, W>;
    type CustomScalarTypeDefinition = ScalarTypeDefinition<'a, S, W>;
    type ObjectTypeDefinition = ObjectTypeDefinition<'a, S, W>;
    type InterfaceTypeDefinition = InterfaceTypeDefinition<'a, S, W>;
    type UnionTypeDefinition = UnionTypeDefinition<'a, S, W>;
    type InputObjectTypeDefinition = InputObjectTypeDefinition<'a, S, W>;
    type EnumTypeDefinition = EnumTypeDefinition<'a, S, W>;
    type TypeDefinition = TypeDefinition<'a, S, W>;
    type DirectiveDefinition = DirectiveDefinition<'a, S, W>;
    type TypeDefinitions<'b> = std::iter::Map<std::slice::Iter<'b, &'b Self::TypeDefinition>, fn(&&'b Self::TypeDefinition) -> definition::TypeDefinitionReference<'b, Self::TypeDefinition>> where 'a: 'b;
    type DirectiveDefinitions<'b> = std::iter::Copied<std::slice::Iter<'b, &'b Self::DirectiveDefinition>> where 'a: 'b;
    type InterfaceImplementors<'b> = std::iter::Copied<std::slice::Iter<'b, &'b Self::ObjectTypeDefinition>> where 'a: 'b;

    fn description(&self) -> Option<&str> {
        self.inner.description()
    }

    fn query(&self) -> &Self::ObjectTypeDefinition {
        &self.query
    }

    fn mutation(&self) -> Option<&Self::ObjectTypeDefinition> {
        self.mutation.as_ref()
    }

    fn subscription(&self) -> Option<&Self::ObjectTypeDefinition> {
        self.subscription.as_ref()
    }

    fn schema_directives(&self) -> Option<&Self::Directives> {
        self.schema_directives.as_ref()
    }

    fn type_definitions(&self) -> Self::TypeDefinitions<'_> {
        self.type_definitions
            .get_or_init(|| {
                self.inner
                    .type_definitions()
                    .filter_map(|tdr| self.cache.get_or_create_type_definition(tdr))
                    .collect()
            })
            .iter()
            .map(|td| definition::TypeDefinition::as_ref(td))
    }

    fn get_type_definition(
        &self,
        name: &str,
    ) -> Option<definition::TypeDefinitionReference<Self::TypeDefinition>> {
        self.cache
            .get_type_definition(name)
            .or_else(|| {
                self.inner
                    .get_type_definition(name)
                    .and_then(|tdr| self.cache.get_or_create_type_definition(tdr))
            })
            .map(TypeDefinition::as_ref)
    }

    fn get_interface_implementors(
        &self,
        itd: &Self::InterfaceTypeDefinition,
    ) -> Self::InterfaceImplementors<'_> {
        self.interface_implementors
            .get(itd.name())
            .map(|ii| ii.iter())
            .unwrap_or_else(|| {
                let interface_implementors = self
                    .inner
                    .get_interface_implementors(itd.inner())
                    .filter_map(|otd| {
                        let otd = self
                            .cache
                            .get_or_create_type_definition(
                                definition::TypeDefinitionReference::Object(otd),
                            )?
                            .as_object()
                            .unwrap();

                        otd.interface_implementations()
                            .map_or(false, |interface_implementations| {
                                interface_implementations
                                    .iter()
                                    .any(|ii| ii.interface().name() == itd.name())
                            })
                            .then_some(otd)
                    })
                    .collect();
                self.interface_implementors
                    .insert(itd.inner().name(), interface_implementors)
                    .iter()
            })
            .copied()
    }

    fn directive_definitions(&self) -> Self::DirectiveDefinitions<'_> {
        self.directive_definitions
            .get_or_init(|| {
                self.inner
                    .directive_definitions()
                    .map(|dd| self.cache.get_or_create_directive_definition(dd))
                    .collect()
            })
            .iter()
            .copied()
    }

    fn get_directive_definition(&self, name: &str) -> Option<&Self::DirectiveDefinition> {
        self.cache.get_directive_definition(name).or_else(|| {
            self.inner
                .get_directive_definition(name)
                .map(|dd| self.cache.get_or_create_directive_definition(dd))
        })
    }
}
