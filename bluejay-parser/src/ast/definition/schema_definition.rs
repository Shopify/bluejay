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
use bluejay_core::definition::SchemaDefinition as CoreSchemaDefinition;
use std::collections::{btree_map::Values, BTreeMap};

#[derive(Debug)]
pub struct SchemaDefinition<'a> {
    type_definitions: BTreeMap<&'a str, &'a TypeDefinitionReference<'a>>,
    directive_definitions: BTreeMap<&'a str, &'a DirectiveDefinition<'a>>,
    description: Option<&'a StringValue>,
    query: &'a ObjectTypeDefinition<'a>,
    mutation: Option<&'a ObjectTypeDefinition<'a>>,
    subscription: Option<&'a ObjectTypeDefinition<'a>>,
    schema_directives: Option<&'a ConstDirectives<'a>>,
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
        Self {
            type_definitions,
            directive_definitions,
            description,
            query,
            mutation,
            subscription,
            schema_directives,
        }
    }
}

impl<'a> CoreSchemaDefinition<'a> for SchemaDefinition<'a> {
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
    type TypeDefinitionReferences =
        std::iter::Copied<Values<'a, &'a str, &'a TypeDefinitionReference<'a>>>;
    type DirectiveDefinitions = std::iter::Copied<Values<'a, &'a str, &'a DirectiveDefinition<'a>>>;

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

    fn get_type_definition(&self, name: &str) -> Option<&Self::TypeDefinitionReference> {
        self.type_definitions.get(name).copied()
    }

    fn type_definitions(&'a self) -> Self::TypeDefinitionReferences {
        self.type_definitions.values().copied()
    }

    fn get_directive_definition(&self, name: &str) -> Option<&Self::DirectiveDefinition> {
        self.directive_definitions.get(name).copied()
    }

    fn directive_definitions(&'a self) -> Self::DirectiveDefinitions {
        self.directive_definitions.values().copied()
    }
}
