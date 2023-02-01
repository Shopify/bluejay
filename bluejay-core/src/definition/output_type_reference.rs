use crate::definition::{
    EnumTypeDefinition, InterfaceTypeDefinition, ObjectTypeDefinition, ScalarTypeDefinition,
    UnionTypeDefinition,
};
use crate::BuiltinScalarDefinition;
use std::marker::PhantomData;

#[derive(Debug)]
pub enum BaseOutputTypeReference<
    CS: ScalarTypeDefinition,
    CSW: AsRef<CS> + Clone,
    E: EnumTypeDefinition,
    EW: AsRef<E> + Clone,
    O: ObjectTypeDefinition,
    OW: AsRef<O> + Clone,
    I: InterfaceTypeDefinition,
    IW: AsRef<I> + Clone,
    U: UnionTypeDefinition,
    UW: AsRef<U> + Clone,
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
        CSW: AsRef<CS> + Clone,
        E: EnumTypeDefinition,
        EW: AsRef<E> + Clone,
        O: ObjectTypeDefinition,
        OW: AsRef<O> + Clone,
        I: InterfaceTypeDefinition,
        IW: AsRef<I> + Clone,
        U: UnionTypeDefinition,
        UW: AsRef<U> + Clone,
    > Clone for BaseOutputTypeReference<CS, CSW, E, EW, O, OW, I, IW, U, UW>
{
    fn clone(&self) -> Self {
        match self {
            Self::BuiltinScalarType(bstd) => Self::BuiltinScalarType(*bstd),
            Self::CustomScalarType(csw, _) => {
                Self::CustomScalarType(csw.clone(), Default::default())
            }
            Self::EnumType(etw, _) => Self::EnumType(etw.clone(), Default::default()),
            Self::ObjectType(otw, _) => Self::ObjectType(otw.clone(), Default::default()),
            Self::InterfaceType(itw, _) => Self::InterfaceType(itw.clone(), Default::default()),
            Self::UnionType(utw, _) => Self::UnionType(utw.clone(), Default::default()),
        }
    }
}

impl<
        CS: ScalarTypeDefinition,
        CSW: AsRef<CS> + Clone,
        E: EnumTypeDefinition,
        EW: AsRef<E> + Clone,
        O: ObjectTypeDefinition,
        OW: AsRef<O> + Clone,
        I: InterfaceTypeDefinition,
        IW: AsRef<I> + Clone,
        U: UnionTypeDefinition,
        UW: AsRef<U> + Clone,
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

pub trait AbstractBaseOutputTypeReference {
    type CustomScalarTypeDefinition: ScalarTypeDefinition;
    type EnumTypeDefinition: EnumTypeDefinition;
    type ObjectTypeDefinition: ObjectTypeDefinition;
    type InterfaceTypeDefinition: InterfaceTypeDefinition;
    type UnionTypeDefinition: UnionTypeDefinition;
    type WrappedCustomScalarTypeDefinition: AsRef<Self::CustomScalarTypeDefinition> + Clone;
    type WrappedEnumTypeDefinition: AsRef<Self::EnumTypeDefinition> + Clone;
    type WrappedObjectTypeDefinition: AsRef<Self::ObjectTypeDefinition> + Clone;
    type WrappedInterfaceTypeDefinition: AsRef<Self::InterfaceTypeDefinition> + Clone;
    type WrappedUnionTypeDefinition: AsRef<Self::UnionTypeDefinition> + Clone;

    fn to_concrete(&self) -> BaseOutputTypeReferenceFromAbstract<Self>;
    fn name(&self) -> &str;
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

    pub fn base(&self) -> &B {
        match self {
            Self::Base(b, _) => b,
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
