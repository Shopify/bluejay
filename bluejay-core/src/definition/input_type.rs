use crate::definition::{
    EnumTypeDefinition, HasDirectives, InputObjectTypeDefinition, ScalarTypeDefinition,
    SchemaDefinition, TypeDefinition, TypeDefinitionReference,
};
use crate::BuiltinScalarDefinition;

#[derive(Debug)]
pub enum BaseInputTypeReference<'a, T: InputType> {
    BuiltinScalar(BuiltinScalarDefinition),
    CustomScalar(&'a T::CustomScalarTypeDefinition),
    InputObject(&'a T::InputObjectTypeDefinition),
    Enum(&'a T::EnumTypeDefinition),
}

impl<T: InputType> Clone for BaseInputTypeReference<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: InputType> Copy for BaseInputTypeReference<'_, T> {}

impl<'a, T: InputType> BaseInputTypeReference<'a, T> {
    pub fn name(&self) -> &'a str {
        match self {
            Self::BuiltinScalar(bstd) => bstd.name(),
            Self::CustomScalar(cstd) => cstd.name(),
            Self::Enum(etd) => etd.name(),
            Self::InputObject(iotd) => iotd.name(),
        }
    }

    pub fn convert<
        I: InputType<
            CustomScalarTypeDefinition = T::CustomScalarTypeDefinition,
            InputObjectTypeDefinition = T::InputObjectTypeDefinition,
            EnumTypeDefinition = T::EnumTypeDefinition,
        >,
    >(
        &self,
    ) -> BaseInputTypeReference<'a, I> {
        match self {
            Self::BuiltinScalar(bstd) => BaseInputTypeReference::BuiltinScalar(*bstd),
            Self::CustomScalar(cstd) => BaseInputTypeReference::CustomScalar(*cstd),
            Self::Enum(etd) => BaseInputTypeReference::Enum(*etd),
            Self::InputObject(iotd) => BaseInputTypeReference::InputObject(*iotd),
        }
    }
}

pub enum InputTypeReference<'a, I: InputType> {
    Base(BaseInputTypeReference<'a, I>, bool),
    List(&'a I, bool),
}

impl<I: InputType> Clone for InputTypeReference<'_, I> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<I: InputType> Copy for InputTypeReference<'_, I> {}

impl<'a, I: InputType> InputTypeReference<'a, I> {
    pub fn is_required(&self) -> bool {
        match self {
            Self::Base(_, r) => *r,
            Self::List(_, r) => *r,
        }
    }

    pub fn base<
        S: SchemaDefinition<
            CustomScalarTypeDefinition = I::CustomScalarTypeDefinition,
            InputObjectTypeDefinition = I::InputObjectTypeDefinition,
            EnumTypeDefinition = I::EnumTypeDefinition,
        >,
    >(
        &self,
        schema_definition: &'a S,
    ) -> BaseInputTypeReference<'a, I> {
        match self {
            Self::Base(b, _) => *b,
            Self::List(l, _) => l.base(schema_definition),
        }
    }

    pub fn unwrap_nullable(&self) -> Self {
        match self {
            Self::Base(b, _) => Self::Base(*b, false),
            Self::List(l, _) => Self::List(l, false),
        }
    }
}

#[derive(Clone)]
pub enum ShallowInputTypeReference<'a, I: InputType> {
    Base(&'a str, bool),
    List(&'a I, bool),
}

impl<I: InputType> ShallowInputTypeReference<'_, I> {
    pub fn is_required(&self) -> bool {
        match self {
            Self::Base(_, r) => *r,
            Self::List(_, r) => *r,
        }
    }
}

impl<I: InputType> std::fmt::Display for ShallowInputTypeReference<'_, I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShallowInputTypeReference::Base(name, required) => {
                write!(f, "{}{}", name, if *required { "!" } else { "" })
            }
            ShallowInputTypeReference::List(inner, required) => {
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

impl<I: InputType> PartialEq for ShallowInputTypeReference<'_, I> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                ShallowInputTypeReference::Base(name1, required1),
                ShallowInputTypeReference::Base(name2, required2),
            ) => required1 == required2 && name1 == name2,
            (
                ShallowInputTypeReference::List(inner1, required1),
                ShallowInputTypeReference::List(inner2, required2),
            ) => required1 == required2 && inner1.as_shallow_ref() == inner2.as_shallow_ref(),
            _ => false,
        }
    }
}

pub trait InputType: Sized {
    type CustomScalarTypeDefinition: ScalarTypeDefinition;
    type InputObjectTypeDefinition: InputObjectTypeDefinition<
        Directives = <Self::CustomScalarTypeDefinition as HasDirectives>::Directives,
    >;
    type EnumTypeDefinition: EnumTypeDefinition<
        Directives = <Self::CustomScalarTypeDefinition as HasDirectives>::Directives,
    >;

    fn as_ref<
        'a,
        S: SchemaDefinition<
            CustomScalarTypeDefinition = Self::CustomScalarTypeDefinition,
            InputObjectTypeDefinition = Self::InputObjectTypeDefinition,
            EnumTypeDefinition = Self::EnumTypeDefinition,
        >,
    >(
        &'a self,
        schema_definition: &'a S,
    ) -> InputTypeReference<'a, Self>;

    fn as_shallow_ref(&self) -> ShallowInputTypeReference<'_, Self>;

    fn display_name(&self) -> String {
        self.as_shallow_ref().to_string()
    }

    fn is_required(&self) -> bool {
        self.as_shallow_ref().is_required()
    }

    fn base<
        'a,
        S: SchemaDefinition<
            CustomScalarTypeDefinition = Self::CustomScalarTypeDefinition,
            InputObjectTypeDefinition = Self::InputObjectTypeDefinition,
            EnumTypeDefinition = Self::EnumTypeDefinition,
        >,
    >(
        &'a self,
        schema_definition: &'a S,
    ) -> BaseInputTypeReference<'a, Self> {
        self.as_ref(schema_definition).base(schema_definition)
    }
}

impl<
        'a,
        T: TypeDefinition,
        I: InputType<
            CustomScalarTypeDefinition = T::CustomScalarTypeDefinition,
            InputObjectTypeDefinition = T::InputObjectTypeDefinition,
            EnumTypeDefinition = T::EnumTypeDefinition,
        >,
    > TryFrom<TypeDefinitionReference<'a, T>> for BaseInputTypeReference<'a, I>
{
    type Error = ();

    fn try_from(value: TypeDefinitionReference<'a, T>) -> Result<Self, Self::Error> {
        match value {
            TypeDefinitionReference::BuiltinScalar(bstd) => Ok(Self::BuiltinScalar(bstd)),
            TypeDefinitionReference::CustomScalar(cstd) => Ok(Self::CustomScalar(cstd)),
            TypeDefinitionReference::Enum(etd) => Ok(Self::Enum(etd)),
            TypeDefinitionReference::InputObject(iotd) => Ok(Self::InputObject(iotd)),
            TypeDefinitionReference::Interface(_)
            | TypeDefinitionReference::Object(_)
            | TypeDefinitionReference::Union(_) => Err(()),
        }
    }
}

impl<'a, I: InputType> HasDirectives for BaseInputTypeReference<'a, I> {
    type Directives = <I::CustomScalarTypeDefinition as HasDirectives>::Directives;

    fn directives(&self) -> Option<&'a Self::Directives> {
        match self {
            Self::BuiltinScalar(_) => None,
            Self::CustomScalar(cstd) => cstd.directives(),
            Self::Enum(etd) => etd.directives(),
            Self::InputObject(iotd) => iotd.directives(),
        }
    }
}
