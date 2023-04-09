use crate::definition::{EnumTypeDefinition, InputObjectTypeDefinition, ScalarTypeDefinition};
use crate::BuiltinScalarDefinition;

#[derive(Debug)]
pub enum BaseInputTypeReference<
    'a,
    CS: ScalarTypeDefinition,
    I: InputObjectTypeDefinition,
    E: EnumTypeDefinition,
> {
    BuiltinScalarType(BuiltinScalarDefinition),
    CustomScalarType(&'a CS),
    InputObjectType(&'a I),
    EnumType(&'a E),
}

impl<'a, CS: ScalarTypeDefinition, I: InputObjectTypeDefinition, E: EnumTypeDefinition> Clone
    for BaseInputTypeReference<'a, CS, I, E>
{
    fn clone(&self) -> Self {
        match self {
            Self::BuiltinScalarType(bstd) => Self::BuiltinScalarType(*bstd),
            Self::CustomScalarType(cstd) => Self::CustomScalarType(*cstd),
            Self::InputObjectType(iotd) => Self::InputObjectType(*iotd),
            Self::EnumType(etd) => Self::EnumType(*etd),
        }
    }
}

impl<'a, CS: ScalarTypeDefinition, I: InputObjectTypeDefinition, E: EnumTypeDefinition> Copy
    for BaseInputTypeReference<'a, CS, I, E>
{
}

impl<'a, CS: ScalarTypeDefinition, I: InputObjectTypeDefinition, E: EnumTypeDefinition>
    BaseInputTypeReference<'a, CS, I, E>
{
    pub fn name(&self) -> &str {
        match self {
            Self::BuiltinScalarType(bstd) => bstd.name(),
            Self::CustomScalarType(cstd) => cstd.name(),
            Self::EnumType(etd) => etd.name(),
            Self::InputObjectType(iotd) => iotd.name(),
        }
    }
}

pub type BaseInputTypeReferenceFromAbstract<'a, T> = BaseInputTypeReference<
    'a,
    <T as AbstractBaseInputTypeReference>::CustomScalarTypeDefinition,
    <T as AbstractBaseInputTypeReference>::InputObjectTypeDefinition,
    <T as AbstractBaseInputTypeReference>::EnumTypeDefinition,
>;

pub trait AbstractBaseInputTypeReference {
    type CustomScalarTypeDefinition: ScalarTypeDefinition;
    type InputObjectTypeDefinition: InputObjectTypeDefinition;
    type EnumTypeDefinition: EnumTypeDefinition;

    fn as_ref(&self) -> BaseInputTypeReferenceFromAbstract<'_, Self>;
}

#[derive(Debug)]
pub enum InputTypeReference<
    'a,
    B: AbstractBaseInputTypeReference,
    I: AbstractInputTypeReference<BaseInputTypeReference = B>,
> {
    Base(&'a B, bool),
    List(&'a I, bool),
}

impl<
        'a,
        B: AbstractBaseInputTypeReference,
        I: AbstractInputTypeReference<BaseInputTypeReference = B>,
    > Clone for InputTypeReference<'a, B, I>
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
        B: AbstractBaseInputTypeReference,
        I: AbstractInputTypeReference<BaseInputTypeReference = B>,
    > Copy for InputTypeReference<'a, B, I>
{
}

impl<
        'a,
        B: AbstractBaseInputTypeReference,
        I: AbstractInputTypeReference<BaseInputTypeReference = B>,
    > InputTypeReference<'a, B, I>
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

pub trait AbstractInputTypeReference: Sized {
    type BaseInputTypeReference: AbstractBaseInputTypeReference;

    fn as_ref(&self) -> InputTypeReferenceFromAbstract<'_, Self>;
}

pub type InputTypeReferenceFromAbstract<'a, T> =
    InputTypeReference<'a, <T as AbstractInputTypeReference>::BaseInputTypeReference, T>;
