use crate::ast::definition::{
    ArgumentsDefinition, BaseInputTypeReference, BaseOutputTypeReference,
    CustomScalarTypeDefinition, DirectiveDefinition, EnumTypeDefinition, EnumValueDefinition,
    EnumValueDefinitions, FieldDefinition, FieldsDefinition, InputFieldsDefinition,
    InputObjectTypeDefinition, InputTypeReference, InputValueDefinition, InterfaceImplementation,
    InterfaceImplementations, InterfaceTypeDefinition, ObjectTypeDefinition, OutputTypeReference,
    TypeDefinitionReference, UnionMemberType, UnionMemberTypes, UnionTypeDefinition,
};
use crate::ast::ConstDirectives;
use crate::lexical_token::StringValue;
use bluejay_core::definition::{
    AbstractTypeDefinitionReference, InterfaceImplementation as CoreInterfaceImplementation,
    ObjectTypeDefinition as CoreObjectTypeDefinition, SchemaDefinition as CoreSchemaDefinition,
    TypeDefinitionReferenceFromAbstract,
};
use bluejay_core::AsIter;
use std::collections::{btree_map::Values, BTreeMap, HashMap};

#[derive(Debug)]
pub struct SchemaDefinition<'a> {
    type_definitions: BTreeMap<&'a str, &'a TypeDefinitionReference<'a>>,
    directive_definitions: BTreeMap<&'a str, &'a DirectiveDefinition<'a>>,
    description: Option<&'a StringValue>,
    query: &'a ObjectTypeDefinition<'a>,
    mutation: Option<&'a ObjectTypeDefinition<'a>>,
    subscription: Option<&'a ObjectTypeDefinition<'a>>,
    schema_directives: Option<&'a ConstDirectives<'a>>,
    interface_implementors: HashMap<&'a str, Vec<&'a ObjectTypeDefinition<'a>>>,
}

impl<'a> SchemaDefinition<'a> {
    pub(crate) fn new(
        type_definitions: BTreeMap<&'a str, &'a TypeDefinitionReference<'a>>,
        directive_definitions: BTreeMap<&'a str, &'a DirectiveDefinition<'a>>,
        description: Option<&'a StringValue>,
        query: &'a ObjectTypeDefinition<'a>,
        mutation: Option<&'a ObjectTypeDefinition<'a>>,
        subscription: Option<&'a ObjectTypeDefinition<'a>>,
        schema_directives: Option<&'a ConstDirectives<'a>>,
    ) -> Self {
        let interface_implementors = Self::interface_implementors(&type_definitions);
        Self {
            type_definitions,
            directive_definitions,
            description,
            query,
            mutation,
            subscription,
            schema_directives,
            interface_implementors,
        }
    }

    fn interface_implementors(
        type_definitions: &BTreeMap<&'a str, &'a TypeDefinitionReference<'a>>,
    ) -> HashMap<&'a str, Vec<&'a ObjectTypeDefinition<'a>>> {
        type_definitions.values().fold(
            HashMap::new(),
            |mut interface_implementors, &type_definition| {
                if let TypeDefinitionReference::Object(otd) = type_definition {
                    if let Some(interface_implementations) = otd.interface_implementations() {
                        interface_implementations
                            .iter()
                            .for_each(|interface_implementation| {
                                let itd = interface_implementation.interface();
                                interface_implementors
                                    .entry(itd.name().as_ref())
                                    .or_default()
                                    .push(otd);
                            });
                    }
                }

                interface_implementors
            },
        )
    }
}

impl<'a> CoreSchemaDefinition for SchemaDefinition<'a> {
    type Directives = ConstDirectives<'a>;
    type InputValueDefinition = InputValueDefinition<'a>;
    type InputFieldsDefinition = InputFieldsDefinition<'a>;
    type ArgumentsDefinition = ArgumentsDefinition<'a>;
    type EnumValueDefinition = EnumValueDefinition<'a>;
    type EnumValueDefinitions = EnumValueDefinitions<'a>;
    type FieldDefinition = FieldDefinition<'a>;
    type FieldsDefinition = FieldsDefinition<'a>;
    type InterfaceImplementation = InterfaceImplementation<'a>;
    type InterfaceImplementations = InterfaceImplementations<'a>;
    type UnionMemberType = UnionMemberType<'a>;
    type UnionMemberTypes = UnionMemberTypes<'a>;
    type BaseInputTypeReference = BaseInputTypeReference<'a>;
    type InputTypeReference = InputTypeReference<'a>;
    type BaseOutputTypeReference = BaseOutputTypeReference<'a>;
    type OutputTypeReference = OutputTypeReference<'a>;
    type CustomScalarTypeDefinition = CustomScalarTypeDefinition<'a>;
    type ObjectTypeDefinition = ObjectTypeDefinition<'a>;
    type InterfaceTypeDefinition = InterfaceTypeDefinition<'a>;
    type UnionTypeDefinition = UnionTypeDefinition<'a>;
    type InputObjectTypeDefinition = InputObjectTypeDefinition<'a>;
    type EnumTypeDefinition = EnumTypeDefinition<'a>;
    type TypeDefinitionReference = TypeDefinitionReference<'a>;
    type DirectiveDefinition = DirectiveDefinition<'a>;
    type TypeDefinitionReferences<'b> = std::iter::Map<
        Values<'b, &'b str, &'b TypeDefinitionReference<'a>>,
        fn(&&'b TypeDefinitionReference<'a>) -> TypeDefinitionReferenceFromAbstract<'b, TypeDefinitionReference<'a>>
    > where 'a: 'b;
    type DirectiveDefinitions<'b> =
        std::iter::Copied<Values<'b, &'b str, &'b DirectiveDefinition<'a>>> where 'a: 'b;
    type IterfaceImplementors<'b> = std::iter::Flatten<std::option::IntoIter<std::iter::Copied<std::slice::Iter<'b, &'b ObjectTypeDefinition<'a>>>>> where 'a: 'b;

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
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

    fn schema_directives(&self) -> Option<&Self::Directives> {
        self.schema_directives
    }

    fn get_type_definition(
        &self,
        name: &str,
    ) -> Option<TypeDefinitionReferenceFromAbstract<'_, Self::TypeDefinitionReference>> {
        self.type_definitions.get(name).map(|tdr| tdr.as_ref())
    }

    fn type_definitions(&self) -> Self::TypeDefinitionReferences<'_> {
        self.type_definitions.values().map(|tdr: &&TypeDefinitionReference| -> TypeDefinitionReferenceFromAbstract<'_, TypeDefinitionReference<'a>> {
            tdr.as_ref()
        })
    }

    fn get_directive_definition(&self, name: &str) -> Option<&Self::DirectiveDefinition> {
        self.directive_definitions.get(name).copied()
    }

    fn directive_definitions(&self) -> Self::DirectiveDefinitions<'_> {
        self.directive_definitions.values().copied()
    }

    fn get_interface_implementors(
        &self,
        itd: &Self::InterfaceTypeDefinition,
    ) -> Self::IterfaceImplementors<'_> {
        self.interface_implementors
            .get(itd.name().as_ref())
            .map(|implementors| implementors.iter().copied())
            .into_iter()
            .flatten()
    }
}
