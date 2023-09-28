use crate::definition::{
    ArgumentsDefinition, Directive, DirectiveDefinition, Directives, EnumTypeDefinition,
    EnumValueDefinition, EnumValueDefinitions, FieldDefinition, FieldsDefinition, HasDirectives,
    InputFieldsDefinition, InputObjectTypeDefinition, InputType, InputValueDefinition,
    InterfaceImplementation, InterfaceImplementations, InterfaceTypeDefinition,
    ObjectTypeDefinition, OutputType, ScalarTypeDefinition, TypeDefinition,
    TypeDefinitionReference, UnionMemberType, UnionMemberTypes, UnionTypeDefinition,
};

pub trait SchemaDefinition:
    HasDirectives<Directives = <Self as SchemaDefinition>::Directives>
{
    type Directive: Directive<DirectiveDefinition = Self::DirectiveDefinition>;
    type Directives: Directives<Directive = Self::Directive>;
    type InputValueDefinition: InputValueDefinition<
        InputType = Self::InputType,
        Directives = <Self as SchemaDefinition>::Directives,
    >;
    type InputFieldsDefinition: InputFieldsDefinition<
        InputValueDefinition = Self::InputValueDefinition,
    >;
    type ArgumentsDefinition: ArgumentsDefinition<ArgumentDefinition = Self::InputValueDefinition>;
    type EnumValueDefinition: EnumValueDefinition<
        Directives = <Self as SchemaDefinition>::Directives,
    >;
    type EnumValueDefinitions: EnumValueDefinitions<EnumValueDefinition = Self::EnumValueDefinition>;
    type FieldDefinition: FieldDefinition<
        ArgumentsDefinition = Self::ArgumentsDefinition,
        OutputType = Self::OutputType,
        Directives = <Self as SchemaDefinition>::Directives,
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
    type InputType: InputType<
        CustomScalarTypeDefinition = Self::CustomScalarTypeDefinition,
        InputObjectTypeDefinition = Self::InputObjectTypeDefinition,
        EnumTypeDefinition = Self::EnumTypeDefinition,
    >;
    type OutputType: OutputType<
        CustomScalarTypeDefinition = Self::CustomScalarTypeDefinition,
        EnumTypeDefinition = Self::EnumTypeDefinition,
        ObjectTypeDefinition = Self::ObjectTypeDefinition,
        InterfaceTypeDefinition = Self::InterfaceTypeDefinition,
        UnionTypeDefinition = Self::UnionTypeDefinition,
    >;
    type CustomScalarTypeDefinition: ScalarTypeDefinition<
        Directives = <Self as SchemaDefinition>::Directives,
    >;
    type ObjectTypeDefinition: ObjectTypeDefinition<
        FieldsDefinition = Self::FieldsDefinition,
        InterfaceImplementations = Self::InterfaceImplementations,
        Directives = <Self as SchemaDefinition>::Directives,
    >;
    type InterfaceTypeDefinition: InterfaceTypeDefinition<
        FieldsDefinition = Self::FieldsDefinition,
        InterfaceImplementations = Self::InterfaceImplementations,
        Directives = <Self as SchemaDefinition>::Directives,
    >;
    type UnionTypeDefinition: UnionTypeDefinition<
        UnionMemberTypes = Self::UnionMemberTypes,
        Directives = <Self as SchemaDefinition>::Directives,
        FieldsDefinition = Self::FieldsDefinition,
    >;
    type InputObjectTypeDefinition: InputObjectTypeDefinition<
        InputFieldsDefinition = Self::InputFieldsDefinition,
        Directives = <Self as SchemaDefinition>::Directives,
    >;
    type EnumTypeDefinition: EnumTypeDefinition<
        Directives = <Self as SchemaDefinition>::Directives,
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
