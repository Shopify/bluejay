use crate::definition::{
    EnumTypeDefinition, InterfaceTypeDefinition, ObjectTypeDefinition, ScalarTypeDefinition,
    UnionTypeDefinition,
};
use crate::BuiltinScalarDefinition;
use std::marker::PhantomData;

#[derive(Debug)]
pub enum BaseOutputTypeReference<
    CS: ScalarTypeDefinition,
    CSW: AsRef<CS>,
    E: EnumTypeDefinition,
    EW: AsRef<E>,
    O: ObjectTypeDefinition,
    OW: AsRef<O>,
    I: InterfaceTypeDefinition,
    IW: AsRef<I>,
    U: UnionTypeDefinition,
    UW: AsRef<U>,
> {
    BuiltinScalarType(BuiltinScalarDefinition),
    CustomScalarType(CSW, PhantomData<CS>),
    EnumType(EW, PhantomData<E>),
    ObjectType(OW, PhantomData<O>),
    InterfaceType(IW, PhantomData<I>),
    UnionType(UW, PhantomData<U>),
}

impl<
        CS: ScalarTypeDefinition,
        CSW: AsRef<CS>,
        E: EnumTypeDefinition,
        EW: AsRef<E>,
        O: ObjectTypeDefinition,
        OW: AsRef<O>,
        I: InterfaceTypeDefinition,
        IW: AsRef<I>,
        U: UnionTypeDefinition,
        UW: AsRef<U>,
    > BaseOutputTypeReference<CS, CSW, E, EW, O, OW, I, IW, U, UW>
{
    pub fn name(&self) -> &str {
        match self {
            Self::BuiltinScalarType(bstd) => bstd.name(),
            Self::CustomScalarType(cstd, _) => cstd.as_ref().name(),
            Self::EnumType(etd, _) => etd.as_ref().name(),
            Self::ObjectType(otd, _) => otd.as_ref().name(),
            Self::InterfaceType(itd, _) => itd.as_ref().name(),
            Self::UnionType(utd, _) => utd.as_ref().name(),
        }
    }

    pub(crate) fn is_scalar_or_enum(&self) -> bool {
        matches!(
            self,
            Self::BuiltinScalarType(_) | Self::CustomScalarType(_, _) | Self::EnumType(_, _)
        )
    }
}

impl<
        CS: ScalarTypeDefinition,
        CSW: AsRef<CS>,
        E: EnumTypeDefinition,
        EW: AsRef<E>,
        O: ObjectTypeDefinition,
        OW: AsRef<O>,
        I: InterfaceTypeDefinition,
        IW: AsRef<I>,
        U: UnionTypeDefinition,
        UW: AsRef<U>,
    > AsRef<Self> for BaseOutputTypeReference<CS, CSW, E, EW, O, OW, I, IW, U, UW>
{
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<
        CS: ScalarTypeDefinition,
        CSW: AsRef<CS>,
        E: EnumTypeDefinition,
        EW: AsRef<E>,
        O: ObjectTypeDefinition,
        OW: AsRef<O>,
        I: InterfaceTypeDefinition,
        IW: AsRef<I>,
        U: UnionTypeDefinition,
        UW: AsRef<U>,
    > AbstractBaseOutputTypeReference
    for BaseOutputTypeReference<CS, CSW, E, EW, O, OW, I, IW, U, UW>
{
    type CustomScalarTypeDefinition = CS;
    type EnumTypeDefinition = E;
    type InterfaceTypeDefinition = I;
    type ObjectTypeDefinition = O;
    type UnionTypeDefinition = U;
    type WrappedCustomScalarTypeDefinition = CSW;
    type WrappedEnumTypeDefinition = EW;
    type WrappedInterfaceTypeDefinition = IW;
    type WrappedObjectTypeDefinition = OW;
    type WrappedUnionTypeDefinition = UW;
}

pub type BaseOutputTypeReferenceFromAbstract<T> = BaseOutputTypeReference<
    <T as AbstractBaseOutputTypeReference>::CustomScalarTypeDefinition,
    <T as AbstractBaseOutputTypeReference>::WrappedCustomScalarTypeDefinition,
    <T as AbstractBaseOutputTypeReference>::EnumTypeDefinition,
    <T as AbstractBaseOutputTypeReference>::WrappedEnumTypeDefinition,
    <T as AbstractBaseOutputTypeReference>::ObjectTypeDefinition,
    <T as AbstractBaseOutputTypeReference>::WrappedObjectTypeDefinition,
    <T as AbstractBaseOutputTypeReference>::InterfaceTypeDefinition,
    <T as AbstractBaseOutputTypeReference>::WrappedInterfaceTypeDefinition,
    <T as AbstractBaseOutputTypeReference>::UnionTypeDefinition,
    <T as AbstractBaseOutputTypeReference>::WrappedUnionTypeDefinition,
>;

pub trait AbstractBaseOutputTypeReference:
    AsRef<BaseOutputTypeReferenceFromAbstract<Self>>
{
    type CustomScalarTypeDefinition: ScalarTypeDefinition;
    type EnumTypeDefinition: EnumTypeDefinition;
    type ObjectTypeDefinition: ObjectTypeDefinition;
    type InterfaceTypeDefinition: InterfaceTypeDefinition;
    type UnionTypeDefinition: UnionTypeDefinition;
    type WrappedCustomScalarTypeDefinition: AsRef<Self::CustomScalarTypeDefinition>;
    type WrappedEnumTypeDefinition: AsRef<Self::EnumTypeDefinition>;
    type WrappedObjectTypeDefinition: AsRef<Self::ObjectTypeDefinition>;
    type WrappedInterfaceTypeDefinition: AsRef<Self::InterfaceTypeDefinition>;
    type WrappedUnionTypeDefinition: AsRef<Self::UnionTypeDefinition>;
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

    pub fn base(&self) -> &BaseOutputTypeReferenceFromAbstract<B> {
        match self {
            Self::Base(b, _) => b.as_ref(),
            Self::List(l, _) => l.as_ref().base(),
        }
    }
}

pub trait AbstractOutputTypeReference:
    AsRef<OutputTypeReference<Self::BaseOutputTypeReference, Self::Wrapper>>
{
    type BaseOutputTypeReference: AbstractBaseOutputTypeReference;
    type Wrapper: AsRef<OutputTypeReference<Self::BaseOutputTypeReference, Self::Wrapper>>;
}
