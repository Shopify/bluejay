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
    type UnionTypeDefinition: UnionTypeDefinition<
        FieldsDefinition = <Self::ObjectTypeDefinition as ObjectTypeDefinition>::FieldsDefinition,
    >;
    type InterfaceTypeDefinition: InterfaceTypeDefinition<
        FieldsDefinition = <Self::ObjectTypeDefinition as ObjectTypeDefinition>::FieldsDefinition,
    >;

    fn as_ref(&self) -> TypeDefinitionReference<'_, Self>;
}

impl<'a, T: TypeDefinition> Clone for TypeDefinitionReference<'a, T> {
    fn clone(&self) -> Self {
        *self
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
        match self {
            Self::BuiltinScalar(_) => true,
            Self::Object(otd) => otd.is_builtin(),
            Self::Enum(etd) => etd.is_builtin(),
            Self::CustomScalar(_) | Self::InputObject(_) | Self::Interface(_) | Self::Union(_) => {
                false
            }
        }
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

    pub fn fields_definition(
        &self,
    ) -> Option<&'a <T::ObjectTypeDefinition as ObjectTypeDefinition>::FieldsDefinition> {
        match self {
            Self::Object(otd) => Some(otd.fields_definition()),
            Self::Interface(itd) => Some(itd.fields_definition()),
            Self::Union(utd) => Some(utd.fields_definition()),
            Self::BuiltinScalar(_)
            | Self::CustomScalar(_)
            | Self::Enum(_)
            | Self::InputObject(_) => None,
        }
    }
}
