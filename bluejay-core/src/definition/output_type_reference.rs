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
    > std::clone::Clone for BaseOutputTypeReference<'a, CS, E, O, I, U>
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

    fn get(&self) -> BaseOutputTypeReferenceFromAbstract<'_, Self>;
}

#[derive(Debug, Clone)]
pub enum OutputTypeReference<B: AbstractBaseOutputTypeReference, W: AsRef<Self>> {
    Base(B, bool),
    List(W, bool),
}

impl<B: AbstractBaseOutputTypeReference, W: AsRef<Self>> OutputTypeReference<B, W> {
    pub fn is_required(&self) -> bool {
        match self {
            Self::Base(_, r) => *r,
            Self::List(_, r) => *r,
        }
    }

    pub fn base(&self) -> BaseOutputTypeReferenceFromAbstract<'_, B> {
        match self {
            Self::Base(b, _) => b.get(),
            Self::List(l, _) => l.as_ref().base(),
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            Self::Base(b, required) => {
                format!("{}{}", b.get().name(), if *required { "!" } else { "" })
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

pub trait AbstractOutputTypeReference:
    AsRef<OutputTypeReference<Self::BaseOutputTypeReference, Self::Wrapper>>
{
    type BaseOutputTypeReference: AbstractBaseOutputTypeReference;
    type Wrapper: AsRef<OutputTypeReference<Self::BaseOutputTypeReference, Self::Wrapper>>;
}
