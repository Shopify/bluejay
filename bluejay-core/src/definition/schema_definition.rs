use crate::definition::{
    ArgumentsDefinition, BaseInputType, BaseOutputType, DirectiveDefinition, EnumTypeDefinition,
    EnumValueDefinition, EnumValueDefinitions, FieldDefinition, FieldsDefinition,
    InputFieldsDefinition, InputObjectTypeDefinition, InputType, InputValueDefinition,
    InterfaceImplementation, InterfaceImplementations, InterfaceTypeDefinition,
    ObjectTypeDefinition, OutputType, ScalarTypeDefinition, TypeDefinition,
    TypeDefinitionReference, UnionMemberType, UnionMemberTypes, UnionTypeDefinition,
};
use crate::ConstDirectives;

pub trait SchemaDefinition {
    type Directives: ConstDirectives;
    type InputValueDefinition: InputValueDefinition<
        InputType = Self::InputType,
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
        OutputType = Self::OutputType,
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
    type BaseInputType: BaseInputType<
        CustomScalarTypeDefinition = Self::CustomScalarTypeDefinition,
        InputObjectTypeDefinition = Self::InputObjectTypeDefinition,
        EnumTypeDefinition = Self::EnumTypeDefinition,
    >;
    type InputType: InputType<BaseInputType = Self::BaseInputType>;
    type BaseOutputType: BaseOutputType<
        CustomScalarTypeDefinition = Self::CustomScalarTypeDefinition,
        EnumTypeDefinition = Self::EnumTypeDefinition,
        ObjectTypeDefinition = Self::ObjectTypeDefinition,
        InterfaceTypeDefinition = Self::InterfaceTypeDefinition,
        UnionTypeDefinition = Self::UnionTypeDefinition,
    >;
    type OutputType: OutputType<BaseOutputType = Self::BaseOutputType>;
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
        FieldsDefinition = Self::FieldsDefinition,
    >;
    type InputObjectTypeDefinition: InputObjectTypeDefinition<
        InputFieldsDefinition = Self::InputFieldsDefinition,
        Directives = Self::Directives,
    >;
    type EnumTypeDefinition: EnumTypeDefinition<
        Directives = Self::Directives,
        EnumValueDefinitions = Self::EnumValueDefinitions,
    >;
    type TypeDefinition: TypeDefinition<
        CustomScalarTypeDefinition = Self::CustomScalarTypeDefinition,
        ObjectTypeDefinition = Self::ObjectTypeDefinition,
        InputObjectTypeDefinition = Self::InputObjectTypeDefinition,
        EnumTypeDefinition = Self::EnumTypeDefinition,
        UnionTypeDefinition = Self::UnionTypeDefinition,
        InterfaceTypeDefinition = Self::InterfaceTypeDefinition,
    >;
    type DirectiveDefinition: DirectiveDefinition<ArgumentsDefinition = Self::ArgumentsDefinition>;
    type TypeDefinitions<'a>: Iterator<Item = TypeDefinitionReference<'a, Self::TypeDefinition>>
    where
        Self: 'a;
    type DirectiveDefinitions<'a>: Iterator<Item = &'a Self::DirectiveDefinition>
    where
        Self: 'a;
    type InterfaceImplementors<'a>: Iterator<Item = &'a Self::ObjectTypeDefinition>
    where
        Self: 'a;

    fn description(&self) -> Option<&str>;
    fn query(&self) -> &Self::ObjectTypeDefinition;
    fn mutation(&self) -> Option<&Self::ObjectTypeDefinition>;
    fn subscription(&self) -> Option<&Self::ObjectTypeDefinition>;
    fn schema_directives(&self) -> Option<&Self::Directives>;
    fn get_type_definition(
        &self,
        name: &str,
    ) -> Option<TypeDefinitionReference<Self::TypeDefinition>>;
    fn type_definitions(&self) -> Self::TypeDefinitions<'_>;
    fn get_directive_definition(&self, name: &str) -> Option<&Self::DirectiveDefinition>;
    fn directive_definitions(&self) -> Self::DirectiveDefinitions<'_>;
    fn get_interface_implementors(
        &self,
        itd: &Self::InterfaceTypeDefinition,
    ) -> Self::InterfaceImplementors<'_>;
}
