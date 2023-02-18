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

impl<
        CS: ScalarTypeDefinition,
        CSW: AsRef<CS>,
        I: InputObjectTypeDefinition,
        IW: AsRef<I>,
        E: EnumTypeDefinition,
        EW: AsRef<E>,
    > AsRef<Self> for BaseInputTypeReference<CS, CSW, I, IW, E, EW>
{
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<
        CS: ScalarTypeDefinition,
        CSW: AsRef<CS>,
        I: InputObjectTypeDefinition,
        IW: AsRef<I>,
        E: EnumTypeDefinition,
        EW: AsRef<E>,
    > BaseInputTypeReference<CS, CSW, I, IW, E, EW>
{
    pub fn name(&self) -> &str {
        match self {
            Self::BuiltinScalarType(bstd) => bstd.name(),
            Self::CustomScalarType(cstd, _) => cstd.as_ref().name(),
            Self::EnumType(etd, _) => etd.as_ref().name(),
            Self::InputObjectType(iotd, _) => iotd.as_ref().name(),
        }
    }
}

impl<
        CS: ScalarTypeDefinition,
        CSW: AsRef<CS>,
        I: InputObjectTypeDefinition,
        IW: AsRef<I>,
        E: EnumTypeDefinition,
        EW: AsRef<E>,
    > AbstractBaseInputTypeReference for BaseInputTypeReference<CS, CSW, I, IW, E, EW>
{
    type CustomScalarTypeDefinition = CS;
    type EnumTypeDefinition = E;
    type InputObjectTypeDefinition = I;
    type WrappedCustomScalarTypeDefinition = CSW;
    type WrappedEnumTypeDefinition = EW;
    type WrappedInputObjectTypeDefinition = IW;
}

pub type BaseInputTypeReferenceFromAbstract<T> = BaseInputTypeReference<
    <T as AbstractBaseInputTypeReference>::CustomScalarTypeDefinition,
    <T as AbstractBaseInputTypeReference>::WrappedCustomScalarTypeDefinition,
    <T as AbstractBaseInputTypeReference>::InputObjectTypeDefinition,
    <T as AbstractBaseInputTypeReference>::WrappedInputObjectTypeDefinition,
    <T as AbstractBaseInputTypeReference>::EnumTypeDefinition,
    <T as AbstractBaseInputTypeReference>::WrappedEnumTypeDefinition,
>;

pub trait AbstractBaseInputTypeReference: AsRef<BaseInputTypeReferenceFromAbstract<Self>> {
    type CustomScalarTypeDefinition: ScalarTypeDefinition;
    type InputObjectTypeDefinition: InputObjectTypeDefinition;
    type EnumTypeDefinition: EnumTypeDefinition;
    type WrappedCustomScalarTypeDefinition: AsRef<Self::CustomScalarTypeDefinition>;
    type WrappedInputObjectTypeDefinition: AsRef<Self::InputObjectTypeDefinition>;
    type WrappedEnumTypeDefinition: AsRef<Self::EnumTypeDefinition>;
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

    pub fn base(&self) -> &BaseInputTypeReferenceFromAbstract<B> {
        match self {
            Self::Base(b, _) => b.as_ref(),
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
