use crate::ast::definition::{
    CustomScalarTypeDefinition, EnumTypeDefinition, InputObjectTypeDefinition,
    InterfaceTypeDefinition, ObjectTypeDefinition, UnionTypeDefinition,
};
use crate::lexical_token::Name;
use bluejay_core::definition::{
    AbstractTypeDefinitionReference, TypeDefinitionReference as CoreTypeDefinitionReference,
    TypeDefinitionReferenceFromAbstract,
};
use bluejay_core::BuiltinScalarDefinition;

#[derive(Debug)]
pub enum TypeDefinitionReference<'a> {
    BuiltinScalar(BuiltinScalarDefinition),
    CustomScalar(CustomScalarTypeDefinition<'a>),
    Object(ObjectTypeDefinition<'a>),
    InputObject(InputObjectTypeDefinition<'a>),
    Enum(EnumTypeDefinition<'a>),
    Union(UnionTypeDefinition<'a>),
    Interface(InterfaceTypeDefinition<'a>),
}

impl<'a> TypeDefinitionReference<'a> {
    pub(crate) fn name(&self) -> Option<&Name<'_>> {
        match self {
            Self::BuiltinScalar(_) => None,
            Self::CustomScalar(cstd) => Some(cstd.name()),
            Self::Enum(etd) => Some(etd.name()),
            Self::InputObject(iotd) => Some(iotd.name()),
            Self::Interface(itd) => Some(itd.name()),
            Self::Object(otd) => Some(otd.name()),
            Self::Union(utd) => Some(utd.name()),
        }
    }

    pub(crate) fn name_str(&self) -> &str {
        match self {
            Self::BuiltinScalar(bstd) => bstd.name(),
            Self::CustomScalar(cstd) => cstd.name().as_ref(),
            Self::Enum(etd) => etd.name().as_ref(),
            Self::InputObject(iotd) => iotd.name().as_ref(),
            Self::Interface(itd) => itd.name().as_ref(),
            Self::Object(otd) => otd.name().as_ref(),
            Self::Union(utd) => utd.name().as_ref(),
        }
    }
}

impl<'a> AbstractTypeDefinitionReference for TypeDefinitionReference<'a> {
    type CustomScalarTypeDefinition = CustomScalarTypeDefinition<'a>;
    type ObjectTypeDefinition = ObjectTypeDefinition<'a>;
    type InputObjectTypeDefinition = InputObjectTypeDefinition<'a>;
    type EnumTypeDefinition = EnumTypeDefinition<'a>;
    type UnionTypeDefinition = UnionTypeDefinition<'a>;
    type InterfaceTypeDefinition = InterfaceTypeDefinition<'a>;

    fn get(&self) -> TypeDefinitionReferenceFromAbstract<'_, Self> {
        match self {
            Self::BuiltinScalar(bstd) => CoreTypeDefinitionReference::BuiltinScalarType(*bstd),
            Self::CustomScalar(cstd) => CoreTypeDefinitionReference::CustomScalarType(cstd),
            Self::Object(otd) => CoreTypeDefinitionReference::ObjectType(otd),
            Self::InputObject(xiotd) => CoreTypeDefinitionReference::InputObjectType(xiotd),
            Self::Enum(etd) => CoreTypeDefinitionReference::EnumType(etd),
            Self::Union(utd) => CoreTypeDefinitionReference::UnionType(utd),
            Self::Interface(itd) => CoreTypeDefinitionReference::InterfaceType(itd),
        }
    }
}

impl<'a> From<CustomScalarTypeDefinition<'a>> for TypeDefinitionReference<'a> {
    fn from(value: CustomScalarTypeDefinition<'a>) -> Self {
        Self::CustomScalar(value)
    }
}

impl<'a> From<ObjectTypeDefinition<'a>> for TypeDefinitionReference<'a> {
    fn from(value: ObjectTypeDefinition<'a>) -> Self {
        Self::Object(value)
    }
}

impl<'a> From<InputObjectTypeDefinition<'a>> for TypeDefinitionReference<'a> {
    fn from(value: InputObjectTypeDefinition<'a>) -> Self {
        Self::InputObject(value)
    }
}

impl<'a> From<InterfaceTypeDefinition<'a>> for TypeDefinitionReference<'a> {
    fn from(value: InterfaceTypeDefinition<'a>) -> Self {
        Self::Interface(value)
    }
}

impl<'a> From<EnumTypeDefinition<'a>> for TypeDefinitionReference<'a> {
    fn from(value: EnumTypeDefinition<'a>) -> Self {
        Self::Enum(value)
    }
}

impl<'a> From<UnionTypeDefinition<'a>> for TypeDefinitionReference<'a> {
    fn from(value: UnionTypeDefinition<'a>) -> Self {
        Self::Union(value)
    }
}
