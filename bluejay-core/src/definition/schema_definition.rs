use crate::definition::{
    AbstractBaseInputTypeReference,
    AbstractBaseOutputTypeReference,
    AbstractInputTypeReference,
    AbstractOutputTypeReference,
    ArgumentsDefinition,
    ScalarTypeDefinition,
    EnumTypeDefinition,
    FieldDefinition,
    FieldsDefinition,
    InputFieldsDefinition,
    InputObjectTypeDefinition,
    InputValueDefinition,
    InterfaceImplementation,
    InterfaceImplementations,
    InterfaceTypeDefinition,
    ObjectTypeDefinition,
    UnionMemberType,
    UnionMemberTypes,
    UnionTypeDefinition,
    AbstractTypeDefinitionReference,
    TypeDefinitionReferenceFromAbstract,
};

pub trait SchemaDefinition<'a>: 'a {
    type InputValueDefinition: InputValueDefinition<InputTypeReference=Self::InputTypeReference>;
    type InputFieldsDefinition: InputFieldsDefinition<InputValueDefinition=Self::InputValueDefinition>;
    type ArgumentsDefinition: ArgumentsDefinition<ArgumentDefinition=Self::InputValueDefinition>;
    type FieldDefinition: FieldDefinition<ArgumentsDefinition=Self::ArgumentsDefinition, OutputTypeReference=Self::OutputTypeReference>;
    type FieldsDefinition: FieldsDefinition<FieldDefinition=Self::FieldDefinition>;
    type InterfaceImplementation: InterfaceImplementation<InterfaceTypeDefinition=Self::InterfaceTypeDefinition>;
    type InterfaceImplementations: InterfaceImplementations<InterfaceImplementation=Self::InterfaceImplementation>;
    type UnionMemberType: UnionMemberType<ObjectTypeDefinition=Self::ObjectTypeDefinition>;
    type UnionMemberTypes: UnionMemberTypes<UnionMemberType=Self::UnionMemberType>;
    type BaseInputTypeReference: AbstractBaseInputTypeReference<CustomScalarTypeDefinition=Self::CustomScalarTypeDefinition, InputObjectTypeDefinition=Self::InputObjectTypeDefinition, EnumTypeDefinition=Self::EnumTypeDefinition>;
    type InputTypeReference: AbstractInputTypeReference<BaseInputTypeReference=Self::BaseInputTypeReference>;
    type BaseOutputTypeReference: AbstractBaseOutputTypeReference<CustomScalarTypeDefinition=Self::CustomScalarTypeDefinition, EnumTypeDefinition=Self::EnumTypeDefinition, ObjectTypeDefinition=Self::ObjectTypeDefinition, InterfaceTypeDefinition=Self::InterfaceTypeDefinition, UnionTypeDefinition=Self::UnionTypeDefinition>;
    type OutputTypeReference: AbstractOutputTypeReference<BaseOutputTypeReference=Self::BaseOutputTypeReference>;
    type CustomScalarTypeDefinition: ScalarTypeDefinition;
    type ObjectTypeDefinition: ObjectTypeDefinition<FieldsDefinition=Self::FieldsDefinition, InterfaceImplementations=Self::InterfaceImplementations>;
    type InterfaceTypeDefinition: InterfaceTypeDefinition<FieldsDefinition=Self::FieldsDefinition, InterfaceImplementations=Self::InterfaceImplementations>;
    type UnionTypeDefinition: UnionTypeDefinition<UnionMemberTypes=Self::UnionMemberTypes>;
    type InputObjectTypeDefinition: InputObjectTypeDefinition<InputFieldsDefinition=Self::InputFieldsDefinition>;
    type EnumTypeDefinition: EnumTypeDefinition;
    type TypeDefinitionReference: AbstractTypeDefinitionReference<CustomScalarTypeDefinition=Self::CustomScalarTypeDefinition, ObjectTypeDefinition=Self::ObjectTypeDefinition, InputObjectTypeDefinition=Self::InputObjectTypeDefinition, EnumTypeDefinition=Self::EnumTypeDefinition, UnionTypeDefinition=Self::UnionTypeDefinition, InterfaceTypeDefinition=Self::InterfaceTypeDefinition>;

    fn description(&self) -> Option<&str>;
    fn query(&self) -> &Self::ObjectTypeDefinition;
    fn mutation(&self) -> Option<&Self::ObjectTypeDefinition>;
    fn get_type(&self, name: &str) -> Option<&TypeDefinitionReferenceFromAbstract<Self::TypeDefinitionReference>>;
}
