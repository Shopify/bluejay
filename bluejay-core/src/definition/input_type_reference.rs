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

impl<'a, CS: ScalarTypeDefinition, I: InputObjectTypeDefinition, E: EnumTypeDefinition>
    std::clone::Clone for BaseInputTypeReference<'a, CS, I, E>
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

    fn get(&self) -> BaseInputTypeReferenceFromAbstract<'_, Self>;
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

    pub fn base(&self) -> BaseInputTypeReferenceFromAbstract<B> {
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
