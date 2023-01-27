use std::marker::PhantomData;
use crate::BuiltinScalarDefinition;
use crate::definition::{
    ScalarTypeDefinition,
    InputObjectTypeDefinition,
    EnumTypeDefinition,
};

#[derive(Debug, Clone)]
pub enum BaseInputTypeReference<CS: ScalarTypeDefinition, CSW: AsRef<CS>, I: InputObjectTypeDefinition, IW: AsRef<I>, E: EnumTypeDefinition, EW: AsRef<E>> {
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
pub enum InputTypeReference<CS: ScalarTypeDefinition, CSW: AsRef<CS>, I: InputObjectTypeDefinition, IW: AsRef<I>, E: EnumTypeDefinition, EW: AsRef<E>> {
    Base(BaseInputTypeReference<CS, CSW, I, IW, E, EW>, bool),
    List(Box<Self>, bool),
}

impl<CS: ScalarTypeDefinition, CSW: AsRef<CS>, I: InputObjectTypeDefinition, IW: AsRef<I>, E: EnumTypeDefinition, EW: AsRef<E>> InputTypeReference<CS, CSW, I, IW, E, EW> {
    pub fn is_required(&self) -> bool {
        match self {
            Self::Base(_, r) => *r,
            Self::List(_, r) => *r,
        }
    }
}

pub type InputTypeReferenceFromAbstract<T> = InputTypeReference<
    <<T as AbstractInputTypeReference>::BaseInputTypeReference as AbstractBaseInputTypeReference>::CustomScalarTypeDefinition,
    <<T as AbstractInputTypeReference>::BaseInputTypeReference as AbstractBaseInputTypeReference>::WrappedCustomScalarTypeDefinition,
    <<T as AbstractInputTypeReference>::BaseInputTypeReference as AbstractBaseInputTypeReference>::InputObjectTypeDefinition,
    <<T as AbstractInputTypeReference>::BaseInputTypeReference as AbstractBaseInputTypeReference>::WrappedInputObjectTypeDefinition,
    <<T as AbstractInputTypeReference>::BaseInputTypeReference as AbstractBaseInputTypeReference>::EnumTypeDefinition,
    <<T as AbstractInputTypeReference>::BaseInputTypeReference as AbstractBaseInputTypeReference>::WrappedEnumTypeDefinition,
>;

pub trait AbstractInputTypeReference {
    type BaseInputTypeReference: AbstractBaseInputTypeReference;

    fn to_concrete(&self) -> InputTypeReferenceFromAbstract<Self>;
}
