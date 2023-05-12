use crate::definition::{
    EnumTypeDefinition, InputObjectTypeDefinition, InterfaceTypeDefinition, ObjectTypeDefinition,
    ScalarTypeDefinition, UnionTypeDefinition,
};
use crate::BuiltinScalarDefinition;
use enum_as_inner::EnumAsInner;

#[derive(Debug, EnumAsInner)]
pub enum TypeDefinitionReference<'a, T: TypeDefinition> {
    BuiltinScalarType(BuiltinScalarDefinition),
    CustomScalarType(&'a T::CustomScalarTypeDefinition),
    ObjectType(&'a T::ObjectTypeDefinition),
    InputObjectType(&'a T::InputObjectTypeDefinition),
    EnumType(&'a T::EnumTypeDefinition),
    UnionType(&'a T::UnionTypeDefinition),
    InterfaceType(&'a T::InterfaceTypeDefinition),
}

pub trait TypeDefinition: Sized {
    type CustomScalarTypeDefinition: ScalarTypeDefinition;
    type ObjectTypeDefinition: ObjectTypeDefinition;
    type InputObjectTypeDefinition: InputObjectTypeDefinition;
    type EnumTypeDefinition: EnumTypeDefinition;
    type UnionTypeDefinition: UnionTypeDefinition;
    type InterfaceTypeDefinition: InterfaceTypeDefinition;

    fn as_ref(&self) -> TypeDefinitionReference<'_, Self>;
}

impl<'a, T: TypeDefinition> Clone for TypeDefinitionReference<'a, T> {
    fn clone(&self) -> Self {
        match self {
            Self::BuiltinScalarType(bstd) => Self::BuiltinScalarType(*bstd),
            Self::CustomScalarType(cs) => Self::CustomScalarType(*cs),
            Self::EnumType(et) => Self::EnumType(*et),
            Self::ObjectType(ot) => Self::ObjectType(*ot),
            Self::InterfaceType(it) => Self::InterfaceType(*it),
            Self::UnionType(ut) => Self::UnionType(*ut),
            Self::InputObjectType(iot) => Self::InputObjectType(*iot),
        }
    }
}

impl<'a, T: TypeDefinition> Copy for TypeDefinitionReference<'a, T> {}

impl<'a, T: TypeDefinition> TypeDefinitionReference<'a, T> {
    pub fn name(&self) -> &'a str {
        match self {
            Self::BuiltinScalarType(bsd) => bsd.name(),
            Self::CustomScalarType(cstd) => cstd.name(),
            Self::ObjectType(otd) => otd.name(),
            Self::InputObjectType(iotd) => iotd.name(),
            Self::EnumType(etd) => etd.name(),
            Self::UnionType(utd) => utd.name(),
            Self::InterfaceType(itd) => itd.name(),
        }
    }

    pub fn is_builtin(&self) -> bool {
        matches!(self, Self::BuiltinScalarType(_))
    }

    pub fn is_composite(&self) -> bool {
        matches!(
            self,
            Self::ObjectType(_) | Self::UnionType(_) | Self::InterfaceType(_)
        )
    }

    pub fn is_input(&self) -> bool {
        matches!(
            self,
            Self::BuiltinScalarType(_)
                | Self::CustomScalarType(_)
                | Self::InputObjectType(_)
                | Self::EnumType(_),
        )
    }
}
