use crate::definition::{
    EnumTypeDefinition, InterfaceTypeDefinition, ObjectTypeDefinition, ScalarTypeDefinition,
    UnionTypeDefinition,
};
use crate::BuiltinScalarDefinition;

#[derive(Debug)]
pub enum BaseOutputTypeReference<
    'a,
    CS: ScalarTypeDefinition,
    E: EnumTypeDefinition,
    O: ObjectTypeDefinition,
    I: InterfaceTypeDefinition,
    U: UnionTypeDefinition,
> {
    BuiltinScalarType(BuiltinScalarDefinition),
    CustomScalarType(&'a CS),
    EnumType(&'a E),
    ObjectType(&'a O),
    InterfaceType(&'a I),
    UnionType(&'a U),
}

impl<
        'a,
        CS: ScalarTypeDefinition,
        E: EnumTypeDefinition,
        O: ObjectTypeDefinition,
        I: InterfaceTypeDefinition,
        U: UnionTypeDefinition,
    > BaseOutputTypeReference<'a, CS, E, O, I, U>
{
    pub fn name(&self) -> &str {
        match self {
            Self::BuiltinScalarType(bstd) => bstd.name(),
            Self::CustomScalarType(cstd) => cstd.name(),
            Self::EnumType(etd) => etd.name(),
            Self::ObjectType(otd) => otd.name(),
            Self::InterfaceType(itd) => itd.name(),
            Self::UnionType(utd) => utd.name(),
        }
    }

    pub fn is_scalar_or_enum(&self) -> bool {
        matches!(
            self,
            Self::BuiltinScalarType(_) | Self::CustomScalarType(_) | Self::EnumType(_)
        )
    }
}

impl<
        'a,
        CS: ScalarTypeDefinition,
        E: EnumTypeDefinition,
        O: ObjectTypeDefinition,
        I: InterfaceTypeDefinition,
        U: UnionTypeDefinition,
    > Clone for BaseOutputTypeReference<'a, CS, E, O, I, U>
{
    fn clone(&self) -> Self {
        match self {
            Self::BuiltinScalarType(bstd) => Self::BuiltinScalarType(*bstd),
            Self::CustomScalarType(cstd) => Self::CustomScalarType(*cstd),
            Self::EnumType(etd) => Self::EnumType(*etd),
            Self::ObjectType(otd) => Self::ObjectType(*otd),
            Self::InterfaceType(itd) => Self::InterfaceType(*itd),
            Self::UnionType(utd) => Self::UnionType(*utd),
        }
    }
}

impl<
        'a,
        CS: ScalarTypeDefinition,
        E: EnumTypeDefinition,
        O: ObjectTypeDefinition,
        I: InterfaceTypeDefinition,
        U: UnionTypeDefinition,
    > Copy for BaseOutputTypeReference<'a, CS, E, O, I, U>
{
}

pub type BaseOutputTypeReferenceFromAbstract<'a, T> = BaseOutputTypeReference<
    'a,
    <T as AbstractBaseOutputTypeReference>::CustomScalarTypeDefinition,
    <T as AbstractBaseOutputTypeReference>::EnumTypeDefinition,
    <T as AbstractBaseOutputTypeReference>::ObjectTypeDefinition,
    <T as AbstractBaseOutputTypeReference>::InterfaceTypeDefinition,
    <T as AbstractBaseOutputTypeReference>::UnionTypeDefinition,
>;

pub trait AbstractBaseOutputTypeReference {
    type CustomScalarTypeDefinition: ScalarTypeDefinition;
    type EnumTypeDefinition: EnumTypeDefinition;
    type ObjectTypeDefinition: ObjectTypeDefinition;
    type InterfaceTypeDefinition: InterfaceTypeDefinition;
    type UnionTypeDefinition: UnionTypeDefinition;

    fn as_ref(&self) -> BaseOutputTypeReferenceFromAbstract<'_, Self>;
}

#[derive(Debug)]
pub enum OutputTypeReference<
    'a,
    B: AbstractBaseOutputTypeReference,
    O: AbstractOutputTypeReference<BaseOutputTypeReference = B>,
> {
    Base(&'a B, bool),
    List(&'a O, bool),
}

impl<
        'a,
        B: AbstractBaseOutputTypeReference,
        O: AbstractOutputTypeReference<BaseOutputTypeReference = B>,
    > Clone for OutputTypeReference<'a, B, O>
{
    fn clone(&self) -> Self {
        match self {
            Self::Base(base, required) => Self::Base(*base, *required),
            Self::List(inner, required) => Self::List(*inner, *required),
        }
    }
}

impl<
        'a,
        B: AbstractBaseOutputTypeReference,
        O: AbstractOutputTypeReference<BaseOutputTypeReference = B>,
    > Copy for OutputTypeReference<'a, B, O>
{
}

impl<
        'a,
        B: AbstractBaseOutputTypeReference,
        O: AbstractOutputTypeReference<BaseOutputTypeReference = B>,
    > OutputTypeReference<'a, B, O>
{
    pub fn is_required(&self) -> bool {
        match self {
            Self::Base(_, r) => *r,
            Self::List(_, r) => *r,
        }
    }

    pub fn base(&self) -> &'a B {
        match self {
            Self::Base(b, _) => b,
            Self::List(l, _) => l.as_ref().base(),
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            Self::Base(b, required) => {
                format!("{}{}", b.as_ref().name(), if *required { "!" } else { "" })
            }
            Self::List(inner, required) => {
                format!(
                    "[{}]{}",
                    inner.as_ref().display_name(),
                    if *required { "!" } else { "" }
                )
            }
        }
    }
}

pub trait AbstractOutputTypeReference: Sized {
    type BaseOutputTypeReference: AbstractBaseOutputTypeReference;

    fn as_ref(&self) -> OutputTypeReferenceFromAbstract<'_, Self>;
}

pub type OutputTypeReferenceFromAbstract<'a, T> =
    OutputTypeReference<'a, <T as AbstractOutputTypeReference>::BaseOutputTypeReference, T>;
