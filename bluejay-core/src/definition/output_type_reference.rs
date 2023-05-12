use crate::definition::{
    EnumTypeDefinition, InterfaceTypeDefinition, ObjectTypeDefinition, ScalarTypeDefinition,
    UnionTypeDefinition,
};
use crate::BuiltinScalarDefinition;

#[derive(Debug)]
pub enum BaseOutputTypeReference<'a, B: BaseOutputType> {
    BuiltinScalar(BuiltinScalarDefinition),
    CustomScalar(&'a B::CustomScalarTypeDefinition),
    Enum(&'a B::EnumTypeDefinition),
    Object(&'a B::ObjectTypeDefinition),
    Interface(&'a B::InterfaceTypeDefinition),
    Union(&'a B::UnionTypeDefinition),
}

impl<'a, B: BaseOutputType> BaseOutputTypeReference<'a, B> {
    pub fn name(&self) -> &str {
        match self {
            Self::BuiltinScalar(bstd) => bstd.name(),
            Self::CustomScalar(cstd) => cstd.name(),
            Self::Enum(etd) => etd.name(),
            Self::Object(otd) => otd.name(),
            Self::Interface(itd) => itd.name(),
            Self::Union(utd) => utd.name(),
        }
    }

    pub fn is_scalar_or_enum(&self) -> bool {
        matches!(
            self,
            Self::BuiltinScalar(_) | Self::CustomScalar(_) | Self::Enum(_)
        )
    }
}

impl<'a, B: BaseOutputType> Clone for BaseOutputTypeReference<'a, B> {
    fn clone(&self) -> Self {
        match self {
            Self::BuiltinScalar(bstd) => Self::BuiltinScalar(*bstd),
            Self::CustomScalar(cstd) => Self::CustomScalar(*cstd),
            Self::Enum(etd) => Self::Enum(*etd),
            Self::Object(otd) => Self::Object(*otd),
            Self::Interface(itd) => Self::Interface(*itd),
            Self::Union(utd) => Self::Union(*utd),
        }
    }
}

impl<'a, B: BaseOutputType> Copy for BaseOutputTypeReference<'a, B> {}

pub trait BaseOutputType: Sized {
    type CustomScalarTypeDefinition: ScalarTypeDefinition;
    type EnumTypeDefinition: EnumTypeDefinition;
    type ObjectTypeDefinition: ObjectTypeDefinition;
    type InterfaceTypeDefinition: InterfaceTypeDefinition;
    type UnionTypeDefinition: UnionTypeDefinition;

    fn as_ref(&self) -> BaseOutputTypeReference<'_, Self>;
}

#[derive(Debug)]
pub enum OutputTypeReference<
    'a,
    B: BaseOutputType,
    O: AbstractOutputTypeReference<BaseOutputType = B>,
> {
    Base(&'a B, bool),
    List(&'a O, bool),
}

impl<'a, B: BaseOutputType, O: AbstractOutputTypeReference<BaseOutputType = B>> Clone
    for OutputTypeReference<'a, B, O>
{
    fn clone(&self) -> Self {
        match self {
            Self::Base(base, required) => Self::Base(*base, *required),
            Self::List(inner, required) => Self::List(*inner, *required),
        }
    }
}

impl<'a, B: BaseOutputType, O: AbstractOutputTypeReference<BaseOutputType = B>> Copy
    for OutputTypeReference<'a, B, O>
{
}

impl<'a, B: BaseOutputType, O: AbstractOutputTypeReference<BaseOutputType = B>>
    OutputTypeReference<'a, B, O>
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

pub trait AbstractOutputTypeReference: Sized {
    type BaseOutputType: BaseOutputType;

    fn as_ref(&self) -> OutputTypeReferenceFromAbstract<'_, Self>;
}

pub type OutputTypeReferenceFromAbstract<'a, T> =
    OutputTypeReference<'a, <T as AbstractOutputTypeReference>::BaseOutputType, T>;
