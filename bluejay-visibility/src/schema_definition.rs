use crate::{
    ArgumentsDefinition, Cache, Directive, DirectiveDefinition, Directives, EnumTypeDefinition,
    EnumValueDefinition, EnumValueDefinitions, FieldDefinition, FieldsDefinition,
    InputFieldsDefinition, InputObjectTypeDefinition, InputType, InputValueDefinition,
    InterfaceImplementation, InterfaceImplementations, InterfaceTypeDefinition,
    ObjectTypeDefinition, OutputType, ScalarTypeDefinition, TypeDefinition, UnionMemberType,
    UnionMemberTypes, UnionTypeDefinition, Warden,
};
use bluejay_core::definition::{
    self, prelude::*, DirectiveLocation, HasDirectives, TypeDefinitionReference,
};
use bluejay_core::{AsIter, Directive as _};
use elsa::FrozenMap;
use once_cell::unsync::OnceCell;
use std::collections::{
    btree_map::{self, Entry},
    BTreeMap,
};

#[derive(Debug)]
pub enum SchemaDefinitionError<'a> {
    QueryRootNotVisible,
    NonUniqueTypeDefinitionName(&'a str),
}

pub struct SchemaDefinition<'a, S: definition::SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    cache: &'a Cache<'a, S, W>,
    query: &'a ObjectTypeDefinition<'a, S, W>,
    mutation: Option<&'a ObjectTypeDefinition<'a, S, W>>,
    subscription: Option<&'a ObjectTypeDefinition<'a, S, W>>,
    interface_implementors: FrozenMap<&'a str, Vec<&'a ObjectTypeDefinition<'a, S, W>>>,
    type_definitions_and_directive_definitions:
        OnceCell<TypeDefinitionsAndDirectiveDefinitions<'a, S, W>>,
    directives: Option<Directives<'a, S, W>>,
}

impl<'a, S: definition::SchemaDefinition, W: Warden<SchemaDefinition = S>>
    SchemaDefinition<'a, S, W>
{
    pub fn new(cache: &'a Cache<'a, S, W>) -> Result<Self, SchemaDefinitionError<'a>> {
        let inner = cache.inner_schema_definition();
        Ok(Self {
            cache,
            query: cache
                .get_or_create_type_definition(TypeDefinitionReference::Object(inner.query()))
                .ok_or(SchemaDefinitionError::QueryRootNotVisible)
                .map(|td| {
                    td.as_object()
                        .expect("Error with internal handling of non-unique type names")
                })?,
            mutation: inner.mutation().and_then(|mutation| {
                cache
                    .get_or_create_type_definition(TypeDefinitionReference::Object(mutation))
                    .map(|td| {
                        td.as_object()
                            .expect("Error with internal handling of non-unique type names")
                    })
            }),
            subscription: inner.subscription().and_then(|subscription| {
                cache
                    .get_or_create_type_definition(TypeDefinitionReference::Object(subscription))
                    .map(|td| {
                        td.as_object()
                            .expect("Error with internal handling of non-unique type names")
                    })
            }),
            interface_implementors: FrozenMap::new(),
            type_definitions_and_directive_definitions: OnceCell::new(),
            directives: inner.directives().map(|d| Directives::new(d, cache)),
        })
    }

    pub fn inner(&self) -> &'a S {
        self.cache.inner_schema_definition()
    }

    pub fn cache(&self) -> &'a Cache<'a, S, W> {
        self.cache
    }

    fn type_definitions_and_directive_definitions(
        &self,
    ) -> &TypeDefinitionsAndDirectiveDefinitions<'a, S, W> {
        self.type_definitions_and_directive_definitions
            .get_or_init(|| VisibilityVisitor::visit(self))
    }

    /// Almost identical type signature to `bluejay_core::definition::SchemaDefinition::get_directive_definition`
    /// except for the lifetime on the return type.
    fn get_directive_definition(&self, name: &str) -> Option<&'a DirectiveDefinition<'a, S, W>> {
        self.cache.get_directive_definition(name).or_else(|| {
            self.inner()
                .get_directive_definition(name)
                .and_then(|dd| self.cache.get_or_create_directive_definition(dd))
        })
    }

    /// Almost identical type signature to `bluejay_core::definition::SchemaDefinition::get_interface_implementors`
    /// except for the lifetime on the return type.
    fn get_interface_implementors(
        &self,
        itd: &InterfaceTypeDefinition<'a, S, W>,
    ) -> std::iter::Copied<std::slice::Iter<'_, &'a ObjectTypeDefinition<'a, S, W>>> {
        self.interface_implementors
            .get(itd.name())
            .map(|ii| ii.iter())
            .unwrap_or_else(|| {
                let interface_implementors = self
                    .inner()
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
                            .is_some_and(|interface_implementations| {
                                interface_implementations
                                    .iter()
                                    .any(|ii| ii.name() == itd.name())
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
}

impl<'a, S: definition::SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>>
    definition::SchemaDefinition for SchemaDefinition<'a, S, W>
{
    type Directive = Directive<'a, S, W>;
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
    type InputType = InputType<'a, S, W>;
    type OutputType = OutputType<'a, S, W>;
    type CustomScalarTypeDefinition = ScalarTypeDefinition<'a, S, W>;
    type ObjectTypeDefinition = ObjectTypeDefinition<'a, S, W>;
    type InterfaceTypeDefinition = InterfaceTypeDefinition<'a, S, W>;
    type UnionTypeDefinition = UnionTypeDefinition<'a, S, W>;
    type InputObjectTypeDefinition = InputObjectTypeDefinition<'a, S, W>;
    type EnumTypeDefinition = EnumTypeDefinition<'a, S, W>;
    type TypeDefinition = TypeDefinition<'a, S, W>;
    type DirectiveDefinition = DirectiveDefinition<'a, S, W>;
    type TypeDefinitions<'b>
        = std::iter::Copied<
        btree_map::Values<'b, &'a str, TypeDefinitionReference<'b, Self::TypeDefinition>>,
    >
    where
        'a: 'b;
    type DirectiveDefinitions<'b>
        = std::iter::Copied<btree_map::Values<'b, &'a str, &'b Self::DirectiveDefinition>>
    where
        'a: 'b;
    type InterfaceImplementors<'b>
        = std::iter::Copied<std::slice::Iter<'b, &'b Self::ObjectTypeDefinition>>
    where
        'a: 'b;

    fn description(&self) -> Option<&str> {
        self.inner().description()
    }

    fn query(&self) -> &Self::ObjectTypeDefinition {
        self.query
    }

    fn mutation(&self) -> Option<&Self::ObjectTypeDefinition> {
        self.mutation
    }

    fn subscription(&self) -> Option<&Self::ObjectTypeDefinition> {
        self.subscription
    }

    fn type_definitions(&self) -> Self::TypeDefinitions<'_> {
        self.type_definitions_and_directive_definitions()
            .type_definitions
            .values()
            .copied()
    }

    fn get_type_definition(
        &self,
        name: &str,
    ) -> Option<definition::TypeDefinitionReference<Self::TypeDefinition>> {
        self.cache
            .get_type_definition(name)
            .or_else(|| {
                self.cache
                    .warden()
                    .type_definitions_for_name(self.inner(), name)
                    .find_map(|tdr| self.cache.get_or_create_type_definition(tdr))
            })
            .map(TypeDefinition::as_ref)
    }

    fn get_interface_implementors(
        &self,
        itd: &Self::InterfaceTypeDefinition,
    ) -> Self::InterfaceImplementors<'_> {
        SchemaDefinition::get_interface_implementors(self, itd)
    }

    fn directive_definitions(&self) -> Self::DirectiveDefinitions<'_> {
        self.type_definitions_and_directive_definitions()
            .directive_definitions
            .values()
            .copied()
    }

    fn get_directive_definition(&self, name: &str) -> Option<&Self::DirectiveDefinition> {
        SchemaDefinition::get_directive_definition(self, name)
    }
}

impl<'a, S: definition::SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> HasDirectives
    for SchemaDefinition<'a, S, W>
{
    type Directives = Directives<'a, S, W>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}

struct TypeDefinitionsAndDirectiveDefinitions<
    'a,
    S: definition::SchemaDefinition,
    W: Warden<SchemaDefinition = S>,
> {
    type_definitions: BTreeMap<&'a str, TypeDefinitionReference<'a, TypeDefinition<'a, S, W>>>,
    directive_definitions: BTreeMap<&'a str, &'a DirectiveDefinition<'a, S, W>>,
}

impl<'a, S: definition::SchemaDefinition, W: Warden<SchemaDefinition = S>>
    From<VisibilityVisitor<'a, '_, S, W>> for TypeDefinitionsAndDirectiveDefinitions<'a, S, W>
{
    fn from(value: VisibilityVisitor<'a, '_, S, W>) -> Self {
        let VisibilityVisitor {
            type_definitions,
            directive_definitions,
            ..
        } = value;
        Self {
            type_definitions,
            directive_definitions,
        }
    }
}

struct VisibilityVisitor<'a, 'b, S: definition::SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    schema_definition: &'b SchemaDefinition<'a, S, W>,
    type_definitions: BTreeMap<&'a str, TypeDefinitionReference<'a, TypeDefinition<'a, S, W>>>,
    directive_definitions: BTreeMap<&'a str, &'a DirectiveDefinition<'a, S, W>>,
}

impl<'a, 'b, S: definition::SchemaDefinition, W: Warden<SchemaDefinition = S>>
    VisibilityVisitor<'a, 'b, S, W>
{
    fn visit(
        schema_definition: &'b SchemaDefinition<'a, S, W>,
    ) -> TypeDefinitionsAndDirectiveDefinitions<'a, S, W> {
        let mut instance = Self {
            schema_definition,
            type_definitions: BTreeMap::new(),
            directive_definitions: BTreeMap::new(),
        };

        // all builtin directive definitions are needed for introspection regardless of whether or not they are accessible in the query
        // also any directive definition that can be used in executable documents must be included here as they don't need to be accessible
        // through the schema visit
        schema_definition
            .inner()
            .directive_definitions()
            .for_each(|dd| {
                if dd.is_builtin() || dd.locations().iter().any(DirectiveLocation::is_executable) {
                    if let Some(dd) = schema_definition
                        .cache
                        .get_or_create_directive_definition(dd)
                    {
                        instance.visit_directive_definition(dd);
                    }
                }
            });

        instance.visit_type_definition(TypeDefinitionReference::Object(schema_definition.query));

        if let Some(mutation) = schema_definition.mutation {
            instance.visit_type_definition(TypeDefinitionReference::Object(mutation));
        }

        if let Some(subscription) = schema_definition.subscription {
            instance.visit_type_definition(TypeDefinitionReference::Object(subscription));
        }

        instance.into()
    }

    fn visit_type_definition(
        &mut self,
        type_definition: TypeDefinitionReference<'a, TypeDefinition<'a, S, W>>,
    ) {
        if let Entry::Vacant(entry) = self.type_definitions.entry(type_definition.name()) {
            entry.insert(type_definition);

            match type_definition {
                TypeDefinitionReference::BuiltinScalar(_) => {}
                TypeDefinitionReference::CustomScalar(cstd) => {
                    self.visit_custom_scalar_type_definition(cstd)
                }
                TypeDefinitionReference::Object(otd) => self.visit_object_type_definition(otd),
                TypeDefinitionReference::Interface(itd) => {
                    self.visit_interface_type_definition(itd)
                }
                TypeDefinitionReference::Union(utd) => self.visit_union_type_definition(utd),
                TypeDefinitionReference::Enum(etd) => self.visit_enum_type_definition(etd),
                TypeDefinitionReference::InputObject(iotd) => {
                    self.visit_input_object_type_definition(iotd)
                }
            }
        }
    }

    fn visit_custom_scalar_type_definition(
        &mut self,
        custom_scalar_type_definition: &'a ScalarTypeDefinition<'a, S, W>,
    ) {
        self.visit_directives(custom_scalar_type_definition.directives());
    }

    fn visit_object_type_definition(
        &mut self,
        object_type_definition: &'a ObjectTypeDefinition<'a, S, W>,
    ) {
        self.visit_directives(object_type_definition.directives());
        self.visit_fields_definition(object_type_definition.fields_definition());
        if let Some(interface_implementations) = object_type_definition.interface_implementations()
        {
            self.visit_interface_implementations(interface_implementations);
        }
    }

    fn visit_interface_type_definition(
        &mut self,
        interface_type_definition: &'a InterfaceTypeDefinition<'a, S, W>,
    ) {
        self.visit_directives(interface_type_definition.directives());
        self.visit_fields_definition(interface_type_definition.fields_definition());
        if let Some(interface_implementations) =
            interface_type_definition.interface_implementations()
        {
            self.visit_interface_implementations(interface_implementations);
        }
        self.schema_definition
            .get_interface_implementors(interface_type_definition)
            .for_each(|otd| {
                self.visit_type_definition(TypeDefinitionReference::Object(otd));
            })
    }

    fn visit_union_type_definition(
        &mut self,
        union_type_definition: &'a UnionTypeDefinition<'a, S, W>,
    ) {
        self.visit_directives(union_type_definition.directives());
        union_type_definition
            .union_member_types()
            .iter()
            .for_each(|member_type| {
                self.visit_type_definition(TypeDefinitionReference::Object(
                    member_type.member_type(),
                ))
            })
    }

    fn visit_enum_type_definition(
        &mut self,
        enum_type_definition: &'a EnumTypeDefinition<'a, S, W>,
    ) {
        self.visit_directives(enum_type_definition.directives());
        enum_type_definition
            .enum_value_definitions()
            .iter()
            .for_each(|etd| {
                self.visit_directives(etd.directives());
            })
    }

    fn visit_input_object_type_definition(
        &mut self,
        input_object_type_definition: &'a InputObjectTypeDefinition<'a, S, W>,
    ) {
        self.visit_directives(input_object_type_definition.directives());
        input_object_type_definition
            .input_field_definitions()
            .iter()
            .for_each(|ivd| {
                self.visit_input_value_definition(ivd);
            })
    }

    fn visit_directives(&mut self, directives: Option<&'a Directives<'a, S, W>>) {
        if let Some(directives) = directives {
            directives.iter().for_each(|directive| {
                // if this is `None` it means that the schema is invalid, but that is not the responsibility of this visitor
                // and should be added to `bluejay-validator`
                if let Some(directive_definition) = self
                    .schema_definition
                    .get_directive_definition(directive.name())
                {
                    self.visit_directive_definition(directive_definition);
                }
            })
        }
    }

    fn visit_directive_definition(
        &mut self,
        directive_definition: &'a DirectiveDefinition<'a, S, W>,
    ) {
        if let Entry::Vacant(entry) = self
            .directive_definitions
            .entry(directive_definition.name())
        {
            entry.insert(directive_definition);
            if let Some(arguments_definition) = directive_definition.arguments_definition() {
                self.visit_arguments_definition(arguments_definition);
            }
        }
    }

    fn visit_fields_definition(&mut self, fields_definition: &'a FieldsDefinition<'a, S, W>) {
        fields_definition.iter().for_each(|field| {
            self.visit_field_definition(field);
        })
    }

    fn visit_field_definition(&mut self, field_definition: &'a FieldDefinition<'a, S, W>) {
        self.visit_directives(field_definition.directives());
        if let Some(arguments_definition) = field_definition.arguments_definition() {
            self.visit_arguments_definition(arguments_definition);
        }
        let base_type = field_definition.r#type().base();
        self.visit_type_definition(base_type.into())
    }

    fn visit_arguments_definition(
        &mut self,
        arguments_definition: &'a ArgumentsDefinition<'a, S, W>,
    ) {
        arguments_definition.iter().for_each(|ivd| {
            self.visit_input_value_definition(ivd);
        })
    }

    fn visit_input_value_definition(
        &mut self,
        input_value_definition: &'a InputValueDefinition<'a, S, W>,
    ) {
        self.visit_directives(input_value_definition.directives());
        let base_type = input_value_definition.r#type().base();
        self.visit_type_definition(base_type.into())
    }

    fn visit_interface_implementations(
        &mut self,
        interface_implementations: &'a InterfaceImplementations<'a, S, W>,
    ) {
        interface_implementations.iter().for_each(|ii| {
            self.visit_type_definition(TypeDefinitionReference::Interface(ii.interface()));
        })
    }
}
