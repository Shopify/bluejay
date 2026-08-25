use crate::definition::{
    EnumTypeDefinition, InterfaceTypeDefinition, ObjectTypeDefinition, ScalarTypeDefinition,
    SchemaDefinition, TypeDefinitionReference, UnionTypeDefinition,
};
use crate::BuiltinScalarDefinition;

#[derive(Debug)]
pub enum BaseOutputTypeReference<'a, S: SchemaDefinition> {
    BuiltinScalar(BuiltinScalarDefinition),
    CustomScalar(&'a S::CustomScalarTypeDefinition),
    Enum(&'a S::EnumTypeDefinition),
    Object(&'a S::ObjectTypeDefinition),
    Interface(&'a S::InterfaceTypeDefinition),
    Union(&'a S::UnionTypeDefinition),
}

impl<'a, S: SchemaDefinition> BaseOutputTypeReference<'a, S> {
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

    pub fn is_composite(&self) -> bool {
        matches!(self, Self::Object(_) | Self::Interface(_) | Self::Union(_))
    }
}

impl<S: SchemaDefinition> Clone for BaseOutputTypeReference<'_, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: SchemaDefinition> Copy for BaseOutputTypeReference<'_, S> {}

pub enum OutputTypeReference<
    'a,
    S: SchemaDefinition,
    O: OutputType<SchemaDefinition = S> = <S as SchemaDefinition>::OutputType,
> {
    Base(BaseOutputTypeReference<'a, S>, bool),
    List(&'a O, bool),
}

impl<S: SchemaDefinition, O: OutputType<SchemaDefinition = S>> Clone
    for OutputTypeReference<'_, S, O>
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: SchemaDefinition, O: OutputType<SchemaDefinition = S>> Copy
    for OutputTypeReference<'_, S, O>
{
}

impl<'a, S: SchemaDefinition, O: OutputType<SchemaDefinition = S>> OutputTypeReference<'a, S, O> {
    pub fn is_required(&self) -> bool {
        match self {
            Self::Base(_, r) => *r,
            Self::List(_, r) => *r,
        }
    }

    pub fn base(&self, schema_definition: &'a S) -> BaseOutputTypeReference<'a, S> {
        match self {
            Self::Base(b, _) => *b,
            Self::List(l, _) => l.base(schema_definition),
        }
    }
}

#[derive(Clone)]
pub enum ShallowOutputTypeReference<'a, O: OutputType> {
    Base(&'a str, bool),
    List(&'a O, bool),
}

impl<'a, O: OutputType> ShallowOutputTypeReference<'a, O> {
    pub fn is_required(&self) -> bool {
        match self {
            Self::Base(_, r) => *r,
            Self::List(_, r) => *r,
        }
    }

    pub fn base_name(&self) -> &'a str {
        match self {
            Self::Base(b, _) => b,
            Self::List(inner, _) => inner.as_shallow_ref().base_name(),
        }
    }
}

impl<O: OutputType> PartialEq for ShallowOutputTypeReference<'_, O> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                ShallowOutputTypeReference::Base(name1, required1),
                ShallowOutputTypeReference::Base(name2, required2),
            ) => required1 == required2 && name1 == name2,
            (
                ShallowOutputTypeReference::List(inner1, required1),
                ShallowOutputTypeReference::List(inner2, required2),
            ) => required1 == required2 && inner1.as_shallow_ref() == inner2.as_shallow_ref(),
            _ => false,
        }
    }
}

impl<O: OutputType> std::fmt::Display for ShallowOutputTypeReference<'_, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShallowOutputTypeReference::Base(name, required) => {
                write!(f, "{}{}", name, if *required { "!" } else { "" })
            }
            ShallowOutputTypeReference::List(inner, required) => {
                write!(
                    f,
                    "[{}]{}",
                    inner.as_shallow_ref(),
                    if *required { "!" } else { "" }
                )
            }
        }
    }
}

pub trait OutputType: Sized {
    type SchemaDefinition: SchemaDefinition;

    fn as_ref<'a>(
        &'a self,
        schema_definition: &'a Self::SchemaDefinition,
    ) -> OutputTypeReference<'a, Self::SchemaDefinition, Self>;

    fn as_shallow_ref(&self) -> ShallowOutputTypeReference<'_, Self>;

    fn display_name(&self) -> String {
        self.as_shallow_ref().to_string()
    }

    fn is_required(&self) -> bool {
        self.as_shallow_ref().is_required()
    }

    fn base_name(&self) -> &str {
        self.as_shallow_ref().base_name()
    }

    fn base<'a>(
        &'a self,
        schema_definition: &'a Self::SchemaDefinition,
    ) -> BaseOutputTypeReference<'a, Self::SchemaDefinition> {
        self.as_ref(schema_definition).base(schema_definition)
    }
}

impl<'a, S: SchemaDefinition> TryFrom<TypeDefinitionReference<'a, S>>
    for BaseOutputTypeReference<'a, S>
{
    type Error = ();

    fn try_from(value: TypeDefinitionReference<'a, S>) -> Result<Self, Self::Error> {
        match value {
            TypeDefinitionReference::BuiltinScalar(bstd) => Ok(Self::BuiltinScalar(bstd)),
            TypeDefinitionReference::CustomScalar(cstd) => Ok(Self::CustomScalar(cstd)),
            TypeDefinitionReference::Enum(etd) => Ok(Self::Enum(etd)),
            TypeDefinitionReference::Interface(itd) => Ok(Self::Interface(itd)),
            TypeDefinitionReference::Object(otd) => Ok(Self::Object(otd)),
            TypeDefinitionReference::Union(utd) => Ok(Self::Union(utd)),
            TypeDefinitionReference::InputObject(_) => Err(()),
        }
    }
}
