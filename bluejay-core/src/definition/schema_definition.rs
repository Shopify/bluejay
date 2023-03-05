use crate::definition::{
    AbstractBaseInputTypeReference, AbstractBaseOutputTypeReference, AbstractInputTypeReference,
    AbstractOutputTypeReference, AbstractTypeDefinitionReference, ArgumentsDefinition,
    DirectiveDefinition, EnumTypeDefinition, EnumValueDefinition, EnumValueDefinitions,
    FieldDefinition, FieldsDefinition, InputFieldsDefinition, InputObjectTypeDefinition,
    InputValueDefinition, InterfaceImplementation, InterfaceImplementations,
    InterfaceTypeDefinition, ObjectTypeDefinition, ScalarTypeDefinition,
    TypeDefinitionReferenceFromAbstract, UnionMemberType, UnionMemberTypes, UnionTypeDefinition,
};
use crate::ConstDirectives;

pub trait SchemaDefinition<'a>: 'a {
    type Directives: ConstDirectives;
    type InputValueDefinition: InputValueDefinition<
        InputTypeReference = Self::InputTypeReference,
        Directives = Self::Directives,
    >;
    type InputFieldsDefinition: InputFieldsDefinition<
        InputValueDefinition = Self::InputValueDefinition,
    >;
    type ArgumentsDefinition: ArgumentsDefinition<ArgumentDefinition = Self::InputValueDefinition>;
    type EnumValueDefinition: EnumValueDefinition<Directives = Self::Directives>;
    type EnumValueDefinitions: EnumValueDefinitions<EnumValueDefinition = Self::EnumValueDefinition>;
    type FieldDefinition: FieldDefinition<
        ArgumentsDefinition = Self::ArgumentsDefinition,
        OutputTypeReference = Self::OutputTypeReference,
        Directives = Self::Directives,
    >;
    type FieldsDefinition: FieldsDefinition<FieldDefinition = Self::FieldDefinition>;
    type InterfaceImplementation: InterfaceImplementation<
        InterfaceTypeDefinition = Self::InterfaceTypeDefinition,
    >;
    type InterfaceImplementations: InterfaceImplementations<
        InterfaceImplementation = Self::InterfaceImplementation,
    >;
    type UnionMemberType: UnionMemberType<ObjectTypeDefinition = Self::ObjectTypeDefinition>;
    type UnionMemberTypes: UnionMemberTypes<UnionMemberType = Self::UnionMemberType>;
    type BaseInputTypeReference: AbstractBaseInputTypeReference<
        CustomScalarTypeDefinition = Self::CustomScalarTypeDefinition,
        InputObjectTypeDefinition = Self::InputObjectTypeDefinition,
        EnumTypeDefinition = Self::EnumTypeDefinition,
    >;
    type InputTypeReference: AbstractInputTypeReference<
        BaseInputTypeReference = Self::BaseInputTypeReference,
    >;
    type BaseOutputTypeReference: AbstractBaseOutputTypeReference<
        CustomScalarTypeDefinition = Self::CustomScalarTypeDefinition,
        EnumTypeDefinition = Self::EnumTypeDefinition,
        ObjectTypeDefinition = Self::ObjectTypeDefinition,
        InterfaceTypeDefinition = Self::InterfaceTypeDefinition,
        UnionTypeDefinition = Self::UnionTypeDefinition,
    >;
    type OutputTypeReference: AbstractOutputTypeReference<
        BaseOutputTypeReference = Self::BaseOutputTypeReference,
    >;
    type CustomScalarTypeDefinition: ScalarTypeDefinition<Directives = Self::Directives>;
    type ObjectTypeDefinition: ObjectTypeDefinition<
        FieldsDefinition = Self::FieldsDefinition,
        InterfaceImplementations = Self::InterfaceImplementations,
        Directives = Self::Directives,
    >;
    type InterfaceTypeDefinition: InterfaceTypeDefinition<
        FieldsDefinition = Self::FieldsDefinition,
        InterfaceImplementations = Self::InterfaceImplementations,
        Directives = Self::Directives,
    >;
    type UnionTypeDefinition: UnionTypeDefinition<
        UnionMemberTypes = Self::UnionMemberTypes,
        Directives = Self::Directives,
    >;
    type InputObjectTypeDefinition: InputObjectTypeDefinition<
        InputFieldsDefinition = Self::InputFieldsDefinition,
        Directives = Self::Directives,
    >;
    type EnumTypeDefinition: EnumTypeDefinition<
        Directives = Self::Directives,
        EnumValueDefinitions = Self::EnumValueDefinitions,
    >;
    type TypeDefinitionReference: AbstractTypeDefinitionReference<
        CustomScalarTypeDefinition = Self::CustomScalarTypeDefinition,
        ObjectTypeDefinition = Self::ObjectTypeDefinition,
        InputObjectTypeDefinition = Self::InputObjectTypeDefinition,
        EnumTypeDefinition = Self::EnumTypeDefinition,
        UnionTypeDefinition = Self::UnionTypeDefinition,
        InterfaceTypeDefinition = Self::InterfaceTypeDefinition,
    >;
    type DirectiveDefinition: DirectiveDefinition<ArgumentsDefinition = Self::ArgumentsDefinition>;
    type TypeDefinitionReferences: Iterator<
        Item = &'a TypeDefinitionReferenceFromAbstract<Self::TypeDefinitionReference>,
    >;
    type DirectiveDefinitions: Iterator<Item = &'a Self::DirectiveDefinition>;

    fn description(&self) -> Option<&str>;
    fn query(&self) -> &Self::ObjectTypeDefinition;
    fn mutation(&self) -> Option<&Self::ObjectTypeDefinition>;
    fn subscription(&self) -> Option<&Self::ObjectTypeDefinition>;
    fn schema_directives(&self) -> Option<&Self::Directives>;
    fn get_type_definition(
        &self,
        name: &str,
    ) -> Option<&TypeDefinitionReferenceFromAbstract<Self::TypeDefinitionReference>>;
    fn type_definitions(&'a self) -> Self::TypeDefinitionReferences;
    fn get_directive_definition(&self, name: &str) -> Option<&Self::DirectiveDefinition>;
    fn directive_definitions(&'a self) -> Self::DirectiveDefinitions;
}
