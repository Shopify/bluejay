use crate::definition::{
    EnumTypeDefinition, InputObjectTypeDefinition, ScalarTypeDefinition, TypeDefinition,
    TypeDefinitionReference,
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
        *self
    }
}

impl<'a, B: BaseInputType> Copy for BaseInputTypeReference<'a, B> {}

impl<'a, B: BaseInputType> BaseInputTypeReference<'a, B> {
    pub fn name(&self) -> &'a str {
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
pub enum InputTypeReference<'a, I: InputType> {
    Base(&'a I::BaseInputType, bool),
    List(&'a I, bool),
}

impl<'a, I: InputType> Clone for InputTypeReference<'a, I> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, I: InputType> Copy for InputTypeReference<'a, I> {}

impl<'a, I: InputType> InputTypeReference<'a, I> {
    pub fn is_required(&self) -> bool {
        match self {
            Self::Base(_, r) => *r,
            Self::List(_, r) => *r,
        }
    }

    pub fn base(&self) -> &'a I::BaseInputType {
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

pub trait InputType: Sized {
    type BaseInputType: BaseInputType;

    fn as_ref(&self) -> InputTypeReference<'_, Self>;
}

impl<
        'a,
        T: TypeDefinition,
        B: BaseInputType<
            CustomScalarTypeDefinition = T::CustomScalarTypeDefinition,
            InputObjectTypeDefinition = T::InputObjectTypeDefinition,
            EnumTypeDefinition = T::EnumTypeDefinition,
        >,
    > TryFrom<TypeDefinitionReference<'a, T>> for BaseInputTypeReference<'a, B>
{
    type Error = ();

    fn try_from(value: TypeDefinitionReference<'a, T>) -> Result<Self, Self::Error> {
        match value {
            TypeDefinitionReference::BuiltinScalar(bstd) => Ok(Self::BuiltinScalar(bstd)),
            TypeDefinitionReference::CustomScalar(cstd) => Ok(Self::CustomScalar(cstd)),
            TypeDefinitionReference::Enum(etd) => Ok(Self::Enum(etd)),
            TypeDefinitionReference::InputObject(iotd) => Ok(Self::InputObject(iotd)),
            TypeDefinitionReference::Interface(_)
            | TypeDefinitionReference::Object(_)
            | TypeDefinitionReference::Union(_) => Err(()),
        }
    }
}
