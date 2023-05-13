use crate::definition::{
    EnumTypeDefinition, InputObjectTypeDefinition, InterfaceTypeDefinition, ObjectTypeDefinition,
    ScalarTypeDefinition, UnionTypeDefinition,
};
use crate::BuiltinScalarDefinition;
use enum_as_inner::EnumAsInner;

#[derive(Debug, EnumAsInner)]
pub enum TypeDefinitionReference<'a, T: TypeDefinition> {
    BuiltinScalar(BuiltinScalarDefinition),
    CustomScalar(&'a T::CustomScalarTypeDefinition),
    Object(&'a T::ObjectTypeDefinition),
    InputObject(&'a T::InputObjectTypeDefinition),
    Enum(&'a T::EnumTypeDefinition),
    Union(&'a T::UnionTypeDefinition),
    Interface(&'a T::InterfaceTypeDefinition),
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
            Self::BuiltinScalar(bstd) => Self::BuiltinScalar(*bstd),
            Self::CustomScalar(cs) => Self::CustomScalar(*cs),
            Self::Enum(et) => Self::Enum(*et),
            Self::Object(ot) => Self::Object(*ot),
            Self::Interface(it) => Self::Interface(*it),
            Self::Union(ut) => Self::Union(*ut),
            Self::InputObject(iot) => Self::InputObject(*iot),
        }
    }
}

impl<'a, T: TypeDefinition> Copy for TypeDefinitionReference<'a, T> {}

impl<'a, T: TypeDefinition> TypeDefinitionReference<'a, T> {
    pub fn name(&self) -> &'a str {
        match self {
            Self::BuiltinScalar(bsd) => bsd.name(),
            Self::CustomScalar(cstd) => cstd.name(),
            Self::Object(otd) => otd.name(),
            Self::InputObject(iotd) => iotd.name(),
            Self::Enum(etd) => etd.name(),
            Self::Union(utd) => utd.name(),
            Self::Interface(itd) => itd.name(),
        }
    }

    pub fn is_builtin(&self) -> bool {
        matches!(self, Self::BuiltinScalar(_))
    }

    pub fn is_composite(&self) -> bool {
        matches!(self, Self::Object(_) | Self::Union(_) | Self::Interface(_))
    }

    pub fn is_input(&self) -> bool {
        matches!(
            self,
            Self::BuiltinScalar(_) | Self::CustomScalar(_) | Self::InputObject(_) | Self::Enum(_),
        )
    }
}
