use crate::definition::{
    EnumTypeDefinition, InputObjectTypeDefinition, ScalarTypeDefinition, TypeDefinition,
    TypeDefinitionReference,
};
use crate::BuiltinScalarDefinition;

#[derive(Debug)]
pub enum BaseInputTypeReference<'a, T: InputType> {
    BuiltinScalar(BuiltinScalarDefinition),
    CustomScalar(&'a T::CustomScalarTypeDefinition),
    InputObject(&'a T::InputObjectTypeDefinition),
    Enum(&'a T::EnumTypeDefinition),
}

impl<'a, T: InputType> Clone for BaseInputTypeReference<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T: InputType> Copy for BaseInputTypeReference<'a, T> {}

impl<'a, T: InputType> BaseInputTypeReference<'a, T> {
    pub fn name(&self) -> &'a str {
        match self {
            Self::BuiltinScalar(bstd) => bstd.name(),
            Self::CustomScalar(cstd) => cstd.name(),
            Self::Enum(etd) => etd.name(),
            Self::InputObject(iotd) => iotd.name(),
        }
    }

    pub fn convert<
        I: InputType<
            CustomScalarTypeDefinition = T::CustomScalarTypeDefinition,
            InputObjectTypeDefinition = T::InputObjectTypeDefinition,
            EnumTypeDefinition = T::EnumTypeDefinition,
        >,
    >(
        &self,
    ) -> BaseInputTypeReference<'a, I> {
        match self {
            Self::BuiltinScalar(bstd) => BaseInputTypeReference::BuiltinScalar(*bstd),
            Self::CustomScalar(cstd) => BaseInputTypeReference::CustomScalar(*cstd),
            Self::Enum(etd) => BaseInputTypeReference::Enum(*etd),
            Self::InputObject(iotd) => BaseInputTypeReference::InputObject(*iotd),
        }
    }
}

pub enum InputTypeReference<'a, I: InputType> {
    Base(BaseInputTypeReference<'a, I>, bool),
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

    pub fn base(&self) -> BaseInputTypeReference<'a, I> {
        match self {
            Self::Base(b, _) => *b,
            Self::List(l, _) => l.as_ref().base(),
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            Self::Base(b, required) => {
                format!("{}{}", b.name(), if *required { "!" } else { "" })
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
            Self::Base(b, _) => Self::Base(*b, false),
            Self::List(l, _) => Self::List(l, false),
        }
    }
}

pub trait InputType: Sized {
    type CustomScalarTypeDefinition: ScalarTypeDefinition;
    type InputObjectTypeDefinition: InputObjectTypeDefinition;
    type EnumTypeDefinition: EnumTypeDefinition;

    fn as_ref(&self) -> InputTypeReference<'_, Self>;
}

impl<
        'a,
        T: TypeDefinition,
        I: InputType<
            CustomScalarTypeDefinition = T::CustomScalarTypeDefinition,
            InputObjectTypeDefinition = T::InputObjectTypeDefinition,
            EnumTypeDefinition = T::EnumTypeDefinition,
        >,
    > TryFrom<TypeDefinitionReference<'a, T>> for BaseInputTypeReference<'a, I>
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
