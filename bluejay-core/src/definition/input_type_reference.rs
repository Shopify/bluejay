use crate::definition::{EnumTypeDefinition, InputObjectTypeDefinition, ScalarTypeDefinition};
use crate::BuiltinScalarDefinition;
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub enum BaseInputTypeReference<
    CS: ScalarTypeDefinition,
    CSW: AsRef<CS>,
    I: InputObjectTypeDefinition,
    IW: AsRef<I>,
    E: EnumTypeDefinition,
    EW: AsRef<E>,
> {
    BuiltinScalarType(BuiltinScalarDefinition),
    CustomScalarType(CSW, PhantomData<CS>),
    InputObjectType(IW, PhantomData<I>),
    EnumType(EW, PhantomData<E>),
}

pub type BaseInputTypeReferenceFromAbstract<T> = BaseInputTypeReference<
    <T as AbstractBaseInputTypeReference>::CustomScalarTypeDefinition,
    <T as AbstractBaseInputTypeReference>::WrappedCustomScalarTypeDefinition,
    <T as AbstractBaseInputTypeReference>::InputObjectTypeDefinition,
    <T as AbstractBaseInputTypeReference>::WrappedInputObjectTypeDefinition,
    <T as AbstractBaseInputTypeReference>::EnumTypeDefinition,
    <T as AbstractBaseInputTypeReference>::WrappedEnumTypeDefinition,
>;

pub trait AbstractBaseInputTypeReference {
    type CustomScalarTypeDefinition: ScalarTypeDefinition;
    type InputObjectTypeDefinition: InputObjectTypeDefinition;
    type EnumTypeDefinition: EnumTypeDefinition;
    type WrappedCustomScalarTypeDefinition: AsRef<Self::CustomScalarTypeDefinition>;
    type WrappedInputObjectTypeDefinition: AsRef<Self::InputObjectTypeDefinition>;
    type WrappedEnumTypeDefinition: AsRef<Self::EnumTypeDefinition>;

    fn to_concrete(&self) -> BaseInputTypeReferenceFromAbstract<Self>;
}

#[derive(Debug, Clone)]
pub enum InputTypeReference<B: AbstractBaseInputTypeReference, W: AsRef<Self>> {
    Base(B, bool),
    List(W, bool),
}

impl<B: AbstractBaseInputTypeReference, W: AsRef<Self>> InputTypeReference<B, W> {
    pub fn is_required(&self) -> bool {
        match self {
            Self::Base(_, r) => *r,
            Self::List(_, r) => *r,
        }
    }

    pub fn base(&self) -> &B {
        match self {
            Self::Base(b, _) => b,
            Self::List(l, _) => l.as_ref().base(),
        }
    }
}

pub trait AbstractInputTypeReference:
    AsRef<InputTypeReference<Self::BaseInputTypeReference, Self::Wrapper>>
{
    type BaseInputTypeReference: AbstractBaseInputTypeReference;
    type Wrapper: AsRef<InputTypeReference<Self::BaseInputTypeReference, Self::Wrapper>>;
}

impl<B: AbstractBaseInputTypeReference, W: AsRef<InputTypeReference<B, W>>> AsRef<Self>
    for InputTypeReference<B, W>
{
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<B: AbstractBaseInputTypeReference, W: AsRef<InputTypeReference<B, W>>>
    AbstractInputTypeReference for InputTypeReference<B, W>
{
    type BaseInputTypeReference = B;
    type Wrapper = W;
}
