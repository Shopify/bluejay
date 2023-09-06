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
    pub fn name(&self) -> &'a str {
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
        *self
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
pub enum OutputTypeReference<'a, O: OutputType> {
    Base(&'a O::BaseOutputType, bool),
    List(&'a O, bool),
}

impl<'a, O: OutputType> Clone for OutputTypeReference<'a, O> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, O: OutputType> Copy for OutputTypeReference<'a, O> {}

impl<'a, O: OutputType> OutputTypeReference<'a, O> {
    pub fn is_required(&self) -> bool {
        match self {
            Self::Base(_, r) => *r,
            Self::List(_, r) => *r,
        }
    }

    pub fn base(&self) -> &'a O::BaseOutputType {
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

pub trait OutputType: Sized {
    type BaseOutputType: BaseOutputType;

    fn as_ref(&self) -> OutputTypeReference<'_, Self>;
}
