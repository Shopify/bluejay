use crate::{
    Cache, EnumTypeDefinition, InputObjectTypeDefinition, InterfaceTypeDefinition,
    ObjectTypeDefinition, ScalarTypeDefinition, UnionTypeDefinition, Warden,
};
use bluejay_core::definition::{self, SchemaDefinition, TypeDefinitionReference};
use bluejay_core::BuiltinScalarDefinition;
use enum_as_inner::EnumAsInner;

#[derive(EnumAsInner)]
pub enum TypeDefinition<'a, S: SchemaDefinition, W: Warden<SchemaDefinition = S>> {
    BuiltinScalar(BuiltinScalarDefinition),
    CustomScalar(ScalarTypeDefinition<'a, S, W>),
    Object(ObjectTypeDefinition<'a, S, W>),
    Interface(InterfaceTypeDefinition<'a, S, W>),
    InputObject(InputObjectTypeDefinition<'a, S, W>),
    Enum(EnumTypeDefinition<'a, S, W>),
    Union(UnionTypeDefinition<'a, S, W>),
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> TypeDefinition<'a, S, W> {
    pub(crate) fn new(
        type_definition: TypeDefinitionReference<'a, S::TypeDefinition>,
        cache: &'a Cache<'a, S, W>,
    ) -> Option<Self> {
        let warden = cache.warden();
        match type_definition {
            TypeDefinitionReference::BuiltinScalar(bstd) => Some(Self::BuiltinScalar(bstd)),
            TypeDefinitionReference::CustomScalar(cstd) => warden
                .is_custom_scalar_type_definition_visible(cstd)
                .then(|| Self::CustomScalar(ScalarTypeDefinition::new(cstd, cache))),
            TypeDefinitionReference::Object(otd) => warden
                .is_object_type_definition_visible(otd)
                .then(|| Self::Object(ObjectTypeDefinition::new(otd, cache))),
            TypeDefinitionReference::Interface(itd) => warden
                .is_interface_type_definition_visible(itd)
                .then(|| Self::Interface(InterfaceTypeDefinition::new(itd, cache))),
            TypeDefinitionReference::InputObject(iotd) => warden
                .is_input_object_type_definition_visible(iotd)
                .then(|| Self::InputObject(InputObjectTypeDefinition::new(iotd, cache))),
            TypeDefinitionReference::Enum(etd) => warden
                .is_enum_type_definition_visible(etd)
                .then(|| Self::Enum(EnumTypeDefinition::new(etd, cache))),
            TypeDefinitionReference::Union(utd) => warden
                .is_union_type_definition_visible(utd)
                .then(|| Self::Union(UnionTypeDefinition::new(utd, cache))),
        }
    }

    pub(crate) fn inner(&self) -> TypeDefinitionReference<'_, S::TypeDefinition> {
        match self {
            Self::BuiltinScalar(bstd) => TypeDefinitionReference::BuiltinScalar(*bstd),
            Self::CustomScalar(cstd) => TypeDefinitionReference::CustomScalar(cstd.inner()),
            Self::Object(otd) => TypeDefinitionReference::Object(otd.inner()),
            Self::Interface(itd) => TypeDefinitionReference::Interface(itd.inner()),
            Self::InputObject(iotd) => TypeDefinitionReference::InputObject(iotd.inner()),
            Self::Enum(etd) => TypeDefinitionReference::Enum(etd.inner()),
            Self::Union(utd) => TypeDefinitionReference::Union(utd.inner()),
        }
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::TypeDefinition
    for TypeDefinition<'a, S, W>
{
    type ObjectTypeDefinition = ObjectTypeDefinition<'a, S, W>;
    type InputObjectTypeDefinition = InputObjectTypeDefinition<'a, S, W>;
    type CustomScalarTypeDefinition = ScalarTypeDefinition<'a, S, W>;
    type InterfaceTypeDefinition = InterfaceTypeDefinition<'a, S, W>;
    type EnumTypeDefinition = EnumTypeDefinition<'a, S, W>;
    type UnionTypeDefinition = UnionTypeDefinition<'a, S, W>;

    fn as_ref(&self) -> TypeDefinitionReference<'_, Self> {
        match self {
            Self::Object(otd) => TypeDefinitionReference::Object(otd),
            Self::Interface(itd) => TypeDefinitionReference::Interface(itd),
            Self::InputObject(iotd) => TypeDefinitionReference::InputObject(iotd),
            Self::CustomScalar(cstd) => TypeDefinitionReference::CustomScalar(cstd),
            Self::BuiltinScalar(bstd) => TypeDefinitionReference::BuiltinScalar(*bstd),
            Self::Enum(etd) => TypeDefinitionReference::Enum(etd),
            Self::Union(utd) => TypeDefinitionReference::Union(utd),
        }
    }
}
