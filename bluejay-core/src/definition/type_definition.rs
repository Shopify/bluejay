use crate::definition::{
    BaseInputTypeReference, BaseOutputTypeReference, EnumTypeDefinition, HasDirectives,
    InputObjectTypeDefinition, InputType, InterfaceTypeDefinition, ObjectTypeDefinition,
    OutputType, ScalarTypeDefinition, UnionTypeDefinition,
};
use crate::BuiltinScalarDefinition;
use enum_as_inner::EnumAsInner;

#[derive(Debug, EnumAsInner)]
pub enum TypeDefinitionReference<'a, T: TypeDefinition> {
    BuiltinScalar(BuiltinScalarDefinition),
    CustomScalar(&'a T::CustomScalarTypeDefinition),
    Object(&'a T::ObjectTypeDefinition),
    InputObject(&'a T::InputObjectTypeDefinition),
    Enum(&'a T::EnumTypeDefinition),
    Union(&'a T::UnionTypeDefinition),
    Interface(&'a T::InterfaceTypeDefinition),
}

impl<
        'a,
        T: TypeDefinition,
        I: InputType<
            CustomScalarTypeDefinition = T::CustomScalarTypeDefinition,
            InputObjectTypeDefinition = T::InputObjectTypeDefinition,
            EnumTypeDefinition = T::EnumTypeDefinition,
        >,
    > From<BaseInputTypeReference<'a, I>> for TypeDefinitionReference<'a, T>
{
    fn from(value: BaseInputTypeReference<'a, I>) -> Self {
        match value {
            BaseInputTypeReference::BuiltinScalar(bstd) => Self::BuiltinScalar(bstd),
            BaseInputTypeReference::CustomScalar(cstd) => Self::CustomScalar(cstd),
            BaseInputTypeReference::Enum(etd) => Self::Enum(etd),
            BaseInputTypeReference::InputObject(iotd) => Self::InputObject(iotd),
        }
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
    > From<BaseOutputTypeReference<'a, O>> for TypeDefinitionReference<'a, T>
{
    fn from(value: BaseOutputTypeReference<'a, O>) -> Self {
        match value {
            BaseOutputTypeReference::BuiltinScalar(bstd) => Self::BuiltinScalar(bstd),
            BaseOutputTypeReference::CustomScalar(cstd) => Self::CustomScalar(cstd),
            BaseOutputTypeReference::Enum(etd) => Self::Enum(etd),
            BaseOutputTypeReference::Object(otd) => Self::Object(otd),
            BaseOutputTypeReference::Interface(itd) => Self::Interface(itd),
            BaseOutputTypeReference::Union(utd) => Self::Union(utd),
        }
    }
}

pub trait TypeDefinition: Sized {
    type CustomScalarTypeDefinition: ScalarTypeDefinition;
    type ObjectTypeDefinition: ObjectTypeDefinition<
        Directives = <Self::CustomScalarTypeDefinition as HasDirectives>::Directives,
    >;
    type InputObjectTypeDefinition: InputObjectTypeDefinition<
        Directives = <Self::CustomScalarTypeDefinition as HasDirectives>::Directives,
    >;
    type EnumTypeDefinition: EnumTypeDefinition<
        Directives = <Self::CustomScalarTypeDefinition as HasDirectives>::Directives,
    >;
    type UnionTypeDefinition: UnionTypeDefinition<
        FieldsDefinition = <Self::ObjectTypeDefinition as ObjectTypeDefinition>::FieldsDefinition,
        Directives = <Self::CustomScalarTypeDefinition as HasDirectives>::Directives,
    >;
    type InterfaceTypeDefinition: InterfaceTypeDefinition<
        FieldsDefinition = <Self::ObjectTypeDefinition as ObjectTypeDefinition>::FieldsDefinition,
        Directives = <Self::CustomScalarTypeDefinition as HasDirectives>::Directives,
    >;

    fn as_ref(&self) -> TypeDefinitionReference<'_, Self>;
}

impl<T: TypeDefinition> Clone for TypeDefinitionReference<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: TypeDefinition> Copy for TypeDefinitionReference<'_, T> {}

impl<'a, T: TypeDefinition> TypeDefinitionReference<'a, T> {
    pub fn name(&self) -> &'a str {
        match self {
            Self::BuiltinScalar(bsd) => bsd.name(),
            Self::CustomScalar(cstd) => cstd.name(),
            Self::Object(otd) => otd.name(),
            Self::InputObject(iotd) => iotd.name(),
            Self::Enum(etd) => etd.name(),
            Self::Union(utd) => utd.name(),
            Self::Interface(itd) => itd.name(),
        }
    }

    pub fn description(&self) -> Option<&'a str> {
        match self {
            Self::BuiltinScalar(_) => None,
            Self::CustomScalar(cstd) => cstd.description(),
            Self::Object(otd) => otd.description(),
            Self::InputObject(iotd) => iotd.description(),
            Self::Enum(etd) => etd.description(),
            Self::Union(utd) => utd.description(),
            Self::Interface(itd) => itd.description(),
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            TypeDefinitionReference::BuiltinScalar(_) => "SCALAR",
            TypeDefinitionReference::CustomScalar(_) => "SCALAR",
            TypeDefinitionReference::Enum(_) => "ENUM",
            TypeDefinitionReference::InputObject(_) => "INPUT_OBJECT",
            TypeDefinitionReference::Interface(_) => "INTERFACE",
            TypeDefinitionReference::Object(_) => "OBJECT",
            TypeDefinitionReference::Union(_) => "UNION",
        }
    }

    pub fn is_builtin(&self) -> bool {
        match self {
            Self::BuiltinScalar(_) => true,
            Self::Object(otd) => otd.is_builtin(),
            Self::Enum(etd) => etd.is_builtin(),
            Self::CustomScalar(_) | Self::InputObject(_) | Self::Interface(_) | Self::Union(_) => {
                false
            }
        }
    }

    pub fn is_composite(&self) -> bool {
        matches!(self, Self::Object(_) | Self::Union(_) | Self::Interface(_))
    }

    pub fn is_abstract(&self) -> bool {
        matches!(self, Self::Interface(_) | Self::Union(_))
    }

    pub fn is_input(&self) -> bool {
        matches!(
            self,
            Self::BuiltinScalar(_) | Self::CustomScalar(_) | Self::InputObject(_) | Self::Enum(_),
        )
    }

    pub fn fields_definition(
        &self,
    ) -> Option<&'a <T::ObjectTypeDefinition as ObjectTypeDefinition>::FieldsDefinition> {
        match self {
            Self::Object(otd) => Some(otd.fields_definition()),
            Self::Interface(itd) => Some(itd.fields_definition()),
            Self::Union(utd) => Some(utd.fields_definition()),
            Self::BuiltinScalar(_)
            | Self::CustomScalar(_)
            | Self::Enum(_)
            | Self::InputObject(_) => None,
        }
    }
}

impl<'a, T: TypeDefinition> HasDirectives for TypeDefinitionReference<'a, T> {
    type Directives = <T::CustomScalarTypeDefinition as HasDirectives>::Directives;

    fn directives(&self) -> Option<&'a Self::Directives> {
        match self {
            Self::BuiltinScalar(_) => None,
            Self::CustomScalar(cstd) => cstd.directives(),
            Self::Object(otd) => otd.directives(),
            Self::InputObject(iotd) => iotd.directives(),
            Self::Enum(etd) => etd.directives(),
            Self::Union(utd) => utd.directives(),
            Self::Interface(itd) => itd.directives(),
        }
    }
}
