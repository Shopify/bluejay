use crate::ast::definition::{
    CustomScalarTypeDefinition, EnumTypeDefinition, InputObjectTypeDefinition,
    InterfaceTypeDefinition, ObjectTypeDefinition, UnionTypeDefinition,
};
use bluejay_core::definition::TypeDefinitionReference as CoreTypeDefinitionReference;

pub type TypeDefinitionReference<'a> = CoreTypeDefinitionReference<
    CustomScalarTypeDefinition<'a>,
    CustomScalarTypeDefinition<'a>,
    ObjectTypeDefinition<'a>,
    ObjectTypeDefinition<'a>,
    InputObjectTypeDefinition<'a>,
    InputObjectTypeDefinition<'a>,
    EnumTypeDefinition<'a>,
    EnumTypeDefinition<'a>,
    UnionTypeDefinition<'a>,
    UnionTypeDefinition<'a>,
    InterfaceTypeDefinition<'a>,
    InterfaceTypeDefinition<'a>,
>;

impl<'a> From<CustomScalarTypeDefinition<'a>> for TypeDefinitionReference<'a> {
    fn from(value: CustomScalarTypeDefinition<'a>) -> Self {
        Self::CustomScalarType(value, Default::default())
    }
}

impl<'a> From<ObjectTypeDefinition<'a>> for TypeDefinitionReference<'a> {
    fn from(value: ObjectTypeDefinition<'a>) -> Self {
        Self::ObjectType(value, Default::default())
    }
}

impl<'a> From<InputObjectTypeDefinition<'a>> for TypeDefinitionReference<'a> {
    fn from(value: InputObjectTypeDefinition<'a>) -> Self {
        Self::InputObjectType(value, Default::default())
    }
}

impl<'a> From<InterfaceTypeDefinition<'a>> for TypeDefinitionReference<'a> {
    fn from(value: InterfaceTypeDefinition<'a>) -> Self {
        Self::InterfaceType(value, Default::default())
    }
}

impl<'a> From<EnumTypeDefinition<'a>> for TypeDefinitionReference<'a> {
    fn from(value: EnumTypeDefinition<'a>) -> Self {
        Self::EnumType(value, Default::default())
    }
}

impl<'a> From<UnionTypeDefinition<'a>> for TypeDefinitionReference<'a> {
    fn from(value: UnionTypeDefinition<'a>) -> Self {
        Self::UnionType(value, Default::default())
    }
}
