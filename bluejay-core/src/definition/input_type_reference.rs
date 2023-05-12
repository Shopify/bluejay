use crate::definition::{
    EnumTypeDefinition, InputObjectTypeDefinition, InterfaceTypeDefinition, ObjectTypeDefinition,
    ScalarTypeDefinition, TypeDefinitionReference, UnionTypeDefinition,
};
use crate::BuiltinScalarDefinition;

#[derive(Debug)]
pub enum BaseInputTypeReference<'a, B: BaseInputType> {
    BuiltinScalar(BuiltinScalarDefinition),
    CustomScalar(&'a B::CustomScalarTypeDefinition),
    InputObject(&'a B::InputObjectTypeDefinition),
    Enum(&'a B::EnumTypeDefinition),
}

impl<'a, B: BaseInputType> Clone for BaseInputTypeReference<'a, B> {
    fn clone(&self) -> Self {
        match self {
            Self::BuiltinScalar(bstd) => Self::BuiltinScalar(*bstd),
            Self::CustomScalar(cstd) => Self::CustomScalar(*cstd),
            Self::InputObject(iotd) => Self::InputObject(*iotd),
            Self::Enum(etd) => Self::Enum(*etd),
        }
    }
}

impl<'a, B: BaseInputType> Copy for BaseInputTypeReference<'a, B> {}

impl<'a, B: BaseInputType> BaseInputTypeReference<'a, B> {
    pub fn name(&self) -> &str {
        match self {
            Self::BuiltinScalar(bstd) => bstd.name(),
            Self::CustomScalar(cstd) => cstd.name(),
            Self::Enum(etd) => etd.name(),
            Self::InputObject(iotd) => iotd.name(),
        }
    }
}

pub trait BaseInputType: Sized {
    type CustomScalarTypeDefinition: ScalarTypeDefinition;
    type InputObjectTypeDefinition: InputObjectTypeDefinition;
    type EnumTypeDefinition: EnumTypeDefinition;

    fn as_ref(&self) -> BaseInputTypeReference<'_, Self>;
}

impl<'a, B: BaseInputType> BaseInputType for BaseInputTypeReference<'a, B> {
    type CustomScalarTypeDefinition = B::CustomScalarTypeDefinition;
    type InputObjectTypeDefinition = B::InputObjectTypeDefinition;
    type EnumTypeDefinition = B::EnumTypeDefinition;

    fn as_ref(&self) -> BaseInputTypeReference<'_, Self> {
        match self {
            Self::BuiltinScalar(bstd) => BaseInputTypeReference::BuiltinScalar(*bstd),
            Self::CustomScalar(cstd) => BaseInputTypeReference::CustomScalar(*cstd),
            Self::Enum(etd) => BaseInputTypeReference::Enum(*etd),
            Self::InputObject(iotd) => BaseInputTypeReference::InputObject(*iotd),
        }
    }
}

#[derive(Debug)]
pub enum InputTypeReference<'a, B: BaseInputType, I: AbstractInputTypeReference<BaseInputType = B>>
{
    Base(&'a B, bool),
    List(&'a I, bool),
}

impl<'a, B: BaseInputType, I: AbstractInputTypeReference<BaseInputType = B>> Clone
    for InputTypeReference<'a, B, I>
{
    fn clone(&self) -> Self {
        match self {
            Self::Base(base, required) => Self::Base(*base, *required),
            Self::List(inner, required) => Self::List(*inner, *required),
        }
    }
}

impl<'a, B: BaseInputType, I: AbstractInputTypeReference<BaseInputType = B>> Copy
    for InputTypeReference<'a, B, I>
{
}

impl<'a, B: BaseInputType, I: AbstractInputTypeReference<BaseInputType = B>>
    InputTypeReference<'a, B, I>
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

    pub fn unwrap_nullable(&self) -> Self {
        match self {
            Self::Base(b, _) => Self::Base(b, false),
            Self::List(l, _) => Self::List(l, false),
        }
    }
}

pub trait AbstractInputTypeReference: Sized {
    type BaseInputType: BaseInputType;

    fn as_ref(&self) -> InputTypeReferenceFromAbstract<'_, Self>;
}

pub type InputTypeReferenceFromAbstract<'a, T> =
    InputTypeReference<'a, <T as AbstractInputTypeReference>::BaseInputType, T>;

impl<
        'a,
        CS: ScalarTypeDefinition,
        O: ObjectTypeDefinition,
        IO: InputObjectTypeDefinition,
        E: EnumTypeDefinition,
        U: UnionTypeDefinition,
        I: InterfaceTypeDefinition,
        B: BaseInputType<
            CustomScalarTypeDefinition = CS,
            InputObjectTypeDefinition = IO,
            EnumTypeDefinition = E,
        >,
    > TryFrom<TypeDefinitionReference<'a, CS, O, IO, E, U, I>> for BaseInputTypeReference<'a, B>
{
    type Error = ();

    fn try_from(
        value: TypeDefinitionReference<'a, CS, O, IO, E, U, I>,
    ) -> Result<Self, Self::Error> {
        match value {
            TypeDefinitionReference::BuiltinScalarType(bstd) => Ok(Self::BuiltinScalar(bstd)),
            TypeDefinitionReference::CustomScalarType(cstd) => Ok(Self::CustomScalar(cstd)),
            TypeDefinitionReference::EnumType(etd) => Ok(Self::Enum(etd)),
            TypeDefinitionReference::InputObjectType(iotd) => Ok(Self::InputObject(iotd)),
            TypeDefinitionReference::InterfaceType(_)
            | TypeDefinitionReference::ObjectType(_)
            | TypeDefinitionReference::UnionType(_) => Err(()),
        }
    }
}
