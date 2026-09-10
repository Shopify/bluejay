use crate::definition::{
    EnumTypeDefinition, HasDirectives, InputObjectTypeDefinition, ScalarTypeDefinition,
    SchemaDefinition, TypeDefinitionReference,
};
use crate::BuiltinScalarDefinition;

#[derive(Debug)]
pub enum BaseInputTypeReference<'a, S: SchemaDefinition> {
    BuiltinScalar(BuiltinScalarDefinition),
    CustomScalar(&'a S::CustomScalarTypeDefinition),
    InputObject(&'a S::InputObjectTypeDefinition),
    Enum(&'a S::EnumTypeDefinition),
}

impl<S: SchemaDefinition> Clone for BaseInputTypeReference<'_, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: SchemaDefinition> Copy for BaseInputTypeReference<'_, S> {}

impl<'a, S: SchemaDefinition> BaseInputTypeReference<'a, S> {
    pub fn name(&self) -> &'a str {
        match self {
            Self::BuiltinScalar(bstd) => bstd.name(),
            Self::CustomScalar(cstd) => cstd.name(),
            Self::Enum(etd) => etd.name(),
            Self::InputObject(iotd) => iotd.name(),
        }
    }
}

pub enum InputTypeReference<'a, S: SchemaDefinition> {
    Base(BaseInputTypeReference<'a, S>, bool),
    List(&'a S::InputType, bool),
}

impl<S: SchemaDefinition> Clone for InputTypeReference<'_, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: SchemaDefinition> Copy for InputTypeReference<'_, S> {}

impl<'a, S: SchemaDefinition> InputTypeReference<'a, S> {
    pub fn is_required(&self) -> bool {
        match self {
            Self::Base(_, r) => *r,
            Self::List(_, r) => *r,
        }
    }

    pub fn base(&self, schema_definition: &'a S) -> BaseInputTypeReference<'a, S> {
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
pub enum ShallowInputTypeReference<'a, S: SchemaDefinition> {
    Base(&'a str, bool),
    List(&'a S::InputType, bool),
}

impl<S: SchemaDefinition> ShallowInputTypeReference<'_, S> {
    pub fn is_required(&self) -> bool {
        match self {
            Self::Base(_, r) => *r,
            Self::List(_, r) => *r,
        }
    }
}

impl<S: SchemaDefinition> std::fmt::Display for ShallowInputTypeReference<'_, S> {
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

impl<S: SchemaDefinition> PartialEq for ShallowInputTypeReference<'_, S> {
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
    type SchemaDefinition: SchemaDefinition<InputType = Self>;

    fn as_ref<'a>(
        &'a self,
        schema_definition: &'a Self::SchemaDefinition,
    ) -> InputTypeReference<'a, Self::SchemaDefinition>;

    fn as_shallow_ref(&self) -> ShallowInputTypeReference<'_, Self::SchemaDefinition>;

    fn display_name(&self) -> String {
        self.as_shallow_ref().to_string()
    }

    fn is_required(&self) -> bool {
        self.as_shallow_ref().is_required()
    }

    fn base<'a>(
        &'a self,
        schema_definition: &'a Self::SchemaDefinition,
    ) -> BaseInputTypeReference<'a, Self::SchemaDefinition> {
        self.as_ref(schema_definition).base(schema_definition)
    }
}

impl<'a, S: SchemaDefinition> TryFrom<TypeDefinitionReference<'a, S>>
    for BaseInputTypeReference<'a, S>
{
    type Error = ();

    fn try_from(value: TypeDefinitionReference<'a, S>) -> Result<Self, Self::Error> {
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

impl<'a, S: SchemaDefinition> HasDirectives for BaseInputTypeReference<'a, S>
where
    <S as SchemaDefinition>::Directives: 'a,
{
    type Directives = <S as SchemaDefinition>::Directives;

    fn directives(&self) -> Option<&'a Self::Directives> {
        match self {
            Self::BuiltinScalar(_) => None,
            Self::CustomScalar(cstd) => cstd.directives(),
            Self::Enum(etd) => etd.directives(),
            Self::InputObject(iotd) => iotd.directives(),
        }
    }
}
