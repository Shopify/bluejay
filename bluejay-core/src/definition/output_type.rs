use crate::definition::{
    EnumTypeDefinition, InterfaceTypeDefinition, ObjectTypeDefinition, ScalarTypeDefinition,
    SchemaDefinition, TypeDefinition, TypeDefinitionReference, UnionTypeDefinition,
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

    pub fn is_composite(&self) -> bool {
        matches!(self, Self::Object(_) | Self::Interface(_) | Self::Union(_))
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

    pub fn base<
        S: SchemaDefinition<
            CustomScalarTypeDefinition = O::CustomScalarTypeDefinition,
            EnumTypeDefinition = O::EnumTypeDefinition,
            ObjectTypeDefinition = O::ObjectTypeDefinition,
            InterfaceTypeDefinition = O::InterfaceTypeDefinition,
            UnionTypeDefinition = O::UnionTypeDefinition,
        >,
    >(
        &self,
        schema_definition: &'a S,
    ) -> BaseOutputTypeReference<'a, O> {
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

impl<'a, O: OutputType> PartialEq for ShallowOutputTypeReference<'a, O> {
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

impl<'a, O: OutputType> std::fmt::Display for ShallowOutputTypeReference<'a, O> {
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
    type CustomScalarTypeDefinition: ScalarTypeDefinition;
    type EnumTypeDefinition: EnumTypeDefinition;
    type ObjectTypeDefinition: ObjectTypeDefinition;
    type InterfaceTypeDefinition: InterfaceTypeDefinition;
    type UnionTypeDefinition: UnionTypeDefinition;

    fn as_ref<
        'a,
        S: SchemaDefinition<
            CustomScalarTypeDefinition = Self::CustomScalarTypeDefinition,
            EnumTypeDefinition = Self::EnumTypeDefinition,
            ObjectTypeDefinition = Self::ObjectTypeDefinition,
            InterfaceTypeDefinition = Self::InterfaceTypeDefinition,
            UnionTypeDefinition = Self::UnionTypeDefinition,
        >,
    >(
        &'a self,
        schema_definition: &'a S,
    ) -> OutputTypeReference<'a, Self>;

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

    fn base<
        'a,
        S: SchemaDefinition<
            CustomScalarTypeDefinition = Self::CustomScalarTypeDefinition,
            EnumTypeDefinition = Self::EnumTypeDefinition,
            ObjectTypeDefinition = Self::ObjectTypeDefinition,
            InterfaceTypeDefinition = Self::InterfaceTypeDefinition,
            UnionTypeDefinition = Self::UnionTypeDefinition,
        >,
    >(
        &'a self,
        schema_definition: &'a S,
    ) -> BaseOutputTypeReference<'a, Self> {
        self.as_ref(schema_definition).base(schema_definition)
    }
}

impl<
        'a,
        T: TypeDefinition,
        O: OutputType<
            CustomScalarTypeDefinition = T::CustomScalarTypeDefinition,
            EnumTypeDefinition = T::EnumTypeDefinition,
            ObjectTypeDefinition = T::ObjectTypeDefinition,
            InterfaceTypeDefinition = T::InterfaceTypeDefinition,
            UnionTypeDefinition = T::UnionTypeDefinition,
        >,
    > TryFrom<TypeDefinitionReference<'a, T>> for BaseOutputTypeReference<'a, O>
{
    type Error = ();

    fn try_from(value: TypeDefinitionReference<'a, T>) -> Result<Self, Self::Error> {
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
