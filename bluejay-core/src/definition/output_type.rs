use crate::definition::{
    EnumTypeDefinition, InterfaceTypeDefinition, ObjectTypeDefinition, ScalarTypeDefinition,
    UnionTypeDefinition,
};
use crate::BuiltinScalarDefinition;

#[derive(Debug)]
pub enum BaseOutputTypeReference<'a, O: OutputType> {
    BuiltinScalar(BuiltinScalarDefinition),
    CustomScalar(&'a O::CustomScalarTypeDefinition),
    Enum(&'a O::EnumTypeDefinition),
    Object(&'a O::ObjectTypeDefinition),
    Interface(&'a O::InterfaceTypeDefinition),
    Union(&'a O::UnionTypeDefinition),
}

impl<'a, O: OutputType> BaseOutputTypeReference<'a, O> {
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

impl<'a, O: OutputType> Clone for BaseOutputTypeReference<'a, O> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, O: OutputType> Copy for BaseOutputTypeReference<'a, O> {}

pub enum OutputTypeReference<'a, O: OutputType> {
    Base(BaseOutputTypeReference<'a, O>, bool),
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

    pub fn base(&self) -> BaseOutputTypeReference<'a, O> {
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
}

pub trait OutputType: Sized {
    type CustomScalarTypeDefinition: ScalarTypeDefinition;
    type EnumTypeDefinition: EnumTypeDefinition;
    type ObjectTypeDefinition: ObjectTypeDefinition;
    type InterfaceTypeDefinition: InterfaceTypeDefinition;
    type UnionTypeDefinition: UnionTypeDefinition;

    fn as_ref(&self) -> OutputTypeReference<'_, Self>;
}
