use crate::ast::definition::{
    Context, CustomScalarTypeDefinition, EnumTypeDefinition, InputObjectTypeDefinition,
    InterfaceTypeDefinition, ObjectTypeDefinition, SchemaDefinition, UnionTypeDefinition,
};
use crate::lexical_token::{Name, StringValue};
use crate::{HasSpan, Span};
use bluejay_core::definition::{TypeDefinition as CoreTypeDefinition, TypeDefinitionReference};
use bluejay_core::BuiltinScalarDefinition;

#[derive(Debug)]
pub enum TypeDefinition<'a, C: Context> {
    BuiltinScalar(BuiltinScalarDefinition),
    CustomScalar(CustomScalarTypeDefinition<'a, C>),
    Object(ObjectTypeDefinition<'a, C>),
    InputObject(InputObjectTypeDefinition<'a, C>),
    Enum(EnumTypeDefinition<'a, C>),
    Union(UnionTypeDefinition<'a, C>),
    Interface(InterfaceTypeDefinition<'a, C>),
}

impl<'a, C: Context> TypeDefinition<'a, C> {
    pub fn name_token(&self) -> Option<&Name<'a>> {
        match self {
            Self::BuiltinScalar(_) => None,
            Self::CustomScalar(cstd) => Some(cstd.name()),
            Self::Enum(etd) => Some(etd.name()),
            Self::InputObject(iotd) => Some(iotd.name_token()),
            Self::Interface(itd) => Some(itd.name()),
            Self::Object(otd) => Some(otd.name()),
            Self::Union(utd) => Some(utd.name()),
        }
    }

    pub fn description_token(&self) -> Option<&StringValue<'a>> {
        match self {
            Self::BuiltinScalar(_) => None,
            Self::CustomScalar(cstd) => cstd.description_token(),
            Self::Enum(etd) => etd.description_token(),
            Self::InputObject(iotd) => iotd.description_token(),
            Self::Interface(itd) => itd.description_token(),
            Self::Object(otd) => otd.description_token(),
            Self::Union(utd) => utd.description_token(),
        }
    }

    /// Starts at the description, if there is one. For a builtin introspection type, the span is
    /// in Bluejay's own definition source, not in the parsed document.
    pub fn span(&self) -> Option<&Span> {
        match self {
            Self::BuiltinScalar(_) => None,
            Self::CustomScalar(cstd) => Some(cstd.span()),
            Self::Enum(etd) => Some(etd.span()),
            Self::InputObject(iotd) => Some(iotd.span()),
            Self::Interface(itd) => Some(itd.span()),
            Self::Object(otd) => Some(otd.span()),
            Self::Union(utd) => Some(utd.span()),
        }
    }

    pub(crate) fn name(&self) -> &str {
        match self {
            Self::BuiltinScalar(bstd) => bstd.name(),
            Self::CustomScalar(cstd) => cstd.name().as_ref(),
            Self::Enum(etd) => etd.name().as_ref(),
            Self::InputObject(iotd) => iotd.name_token().as_ref(),
            Self::Interface(itd) => itd.name().as_ref(),
            Self::Object(otd) => otd.name().as_ref(),
            Self::Union(utd) => utd.name().as_ref(),
        }
    }
}

impl<'a, C: Context> CoreTypeDefinition for TypeDefinition<'a, C> {
    type SchemaDefinition = SchemaDefinition<'a, C>;

    fn as_ref(&self) -> TypeDefinitionReference<'_, SchemaDefinition<'a, C>> {
        match self {
            Self::BuiltinScalar(bstd) => TypeDefinitionReference::BuiltinScalar(*bstd),
            Self::CustomScalar(cstd) => TypeDefinitionReference::CustomScalar(cstd),
            Self::Object(otd) => TypeDefinitionReference::Object(otd),
            Self::InputObject(iotd) => TypeDefinitionReference::InputObject(iotd),
            Self::Enum(etd) => TypeDefinitionReference::Enum(etd),
            Self::Union(utd) => TypeDefinitionReference::Union(utd),
            Self::Interface(itd) => TypeDefinitionReference::Interface(itd),
        }
    }
}

impl<'a, C: Context> From<CustomScalarTypeDefinition<'a, C>> for TypeDefinition<'a, C> {
    fn from(value: CustomScalarTypeDefinition<'a, C>) -> Self {
        Self::CustomScalar(value)
    }
}

impl<'a, C: Context> From<ObjectTypeDefinition<'a, C>> for TypeDefinition<'a, C> {
    fn from(value: ObjectTypeDefinition<'a, C>) -> Self {
        Self::Object(value)
    }
}

impl<'a, C: Context> From<InputObjectTypeDefinition<'a, C>> for TypeDefinition<'a, C> {
    fn from(value: InputObjectTypeDefinition<'a, C>) -> Self {
        Self::InputObject(value)
    }
}

impl<'a, C: Context> From<InterfaceTypeDefinition<'a, C>> for TypeDefinition<'a, C> {
    fn from(value: InterfaceTypeDefinition<'a, C>) -> Self {
        Self::Interface(value)
    }
}

impl<'a, C: Context> From<EnumTypeDefinition<'a, C>> for TypeDefinition<'a, C> {
    fn from(value: EnumTypeDefinition<'a, C>) -> Self {
        Self::Enum(value)
    }
}

impl<'a, C: Context> From<UnionTypeDefinition<'a, C>> for TypeDefinition<'a, C> {
    fn from(value: UnionTypeDefinition<'a, C>) -> Self {
        Self::Union(value)
    }
}
