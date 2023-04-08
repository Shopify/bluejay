use crate::definition::{
    EnumTypeDefinition, InputObjectTypeDefinition, InterfaceTypeDefinition, ObjectTypeDefinition,
    ScalarTypeDefinition, UnionTypeDefinition,
};
use crate::BuiltinScalarDefinition;

#[derive(Debug)]
pub enum TypeDefinitionReference<
    'a,
    CS: ScalarTypeDefinition,
    O: ObjectTypeDefinition,
    IO: InputObjectTypeDefinition,
    E: EnumTypeDefinition,
    U: UnionTypeDefinition,
    I: InterfaceTypeDefinition,
> {
    BuiltinScalarType(BuiltinScalarDefinition),
    CustomScalarType(&'a CS),
    ObjectType(&'a O),
    InputObjectType(&'a IO),
    EnumType(&'a E),
    UnionType(&'a U),
    InterfaceType(&'a I),
}

pub trait AbstractTypeDefinitionReference {
    type CustomScalarTypeDefinition: ScalarTypeDefinition;
    type ObjectTypeDefinition: ObjectTypeDefinition;
    type InputObjectTypeDefinition: InputObjectTypeDefinition;
    type EnumTypeDefinition: EnumTypeDefinition;
    type UnionTypeDefinition: UnionTypeDefinition;
    type InterfaceTypeDefinition: InterfaceTypeDefinition;

    fn get(&self) -> TypeDefinitionReferenceFromAbstract<'_, Self>;
}

pub type TypeDefinitionReferenceFromAbstract<'a, T> = TypeDefinitionReference<
    'a,
    <T as AbstractTypeDefinitionReference>::CustomScalarTypeDefinition,
    <T as AbstractTypeDefinitionReference>::ObjectTypeDefinition,
    <T as AbstractTypeDefinitionReference>::InputObjectTypeDefinition,
    <T as AbstractTypeDefinitionReference>::EnumTypeDefinition,
    <T as AbstractTypeDefinitionReference>::UnionTypeDefinition,
    <T as AbstractTypeDefinitionReference>::InterfaceTypeDefinition,
>;

impl<
        'a,
        CS: ScalarTypeDefinition,
        O: ObjectTypeDefinition,
        IO: InputObjectTypeDefinition,
        E: EnumTypeDefinition,
        U: UnionTypeDefinition,
        I: InterfaceTypeDefinition,
    > Clone for TypeDefinitionReference<'a, CS, O, IO, E, U, I>
{
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

impl<
        'a,
        CS: ScalarTypeDefinition,
        O: ObjectTypeDefinition,
        IO: InputObjectTypeDefinition,
        E: EnumTypeDefinition,
        U: UnionTypeDefinition,
        I: InterfaceTypeDefinition,
    > Copy for TypeDefinitionReference<'a, CS, O, IO, E, U, I>
{
}

impl<
        'a,
        CS: ScalarTypeDefinition,
        O: ObjectTypeDefinition,
        IO: InputObjectTypeDefinition,
        E: EnumTypeDefinition,
        U: UnionTypeDefinition,
        I: InterfaceTypeDefinition,
    > TypeDefinitionReference<'a, CS, O, IO, E, U, I>
{
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
}
