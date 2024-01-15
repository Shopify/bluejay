use crate::{
    Cache, EnumTypeDefinition, InputObjectTypeDefinition, ScalarTypeDefinition, TypeDefinition,
    Warden,
};
use bluejay_core::definition::{
    self, prelude::*, BaseInputTypeReference, InputTypeReference, SchemaDefinition,
    TypeDefinitionReference,
};

pub enum InputType<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S> + 'a> {
    Base(BaseInputTypeReference<'a, Self>, bool),
    List(Box<Self>, bool),
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S> + 'a> InputType<'a, S, W> {
    pub fn new(inner: &'a S::InputType, cache: &'a Cache<'a, S, W>) -> Option<Self> {
        match inner.as_ref(cache.inner_schema_definition()) {
            InputTypeReference::Base(b, required) => {
                Self::new_base(b, cache).map(|base| Self::Base(base, required))
            }
            InputTypeReference::List(inner, required) => {
                Self::new(inner, cache).map(|inner| Self::List(Box::new(inner), required))
            }
        }
    }

    fn new_base(
        inner: BaseInputTypeReference<'a, S::InputType>,
        cache: &'a Cache<'a, S, W>,
    ) -> Option<BaseInputTypeReference<'a, Self>> {
        let tdr = match inner {
            BaseInputTypeReference::BuiltinScalar(bstd) => {
                TypeDefinitionReference::BuiltinScalar(bstd)
            }
            BaseInputTypeReference::CustomScalar(cstd) => {
                TypeDefinitionReference::CustomScalar(cstd)
            }
            BaseInputTypeReference::Enum(etd) => TypeDefinitionReference::Enum(etd),
            BaseInputTypeReference::InputObject(iotd) => TypeDefinitionReference::InputObject(iotd),
        };

        cache
            .get_or_create_type_definition(tdr)
            .map(|type_definition| match type_definition {
                TypeDefinition::BuiltinScalar(bstd) => BaseInputTypeReference::BuiltinScalar(*bstd),
                TypeDefinition::CustomScalar(cstd) => BaseInputTypeReference::CustomScalar(cstd),
                TypeDefinition::Enum(etd) => BaseInputTypeReference::Enum(etd),
                TypeDefinition::InputObject(iotd) => BaseInputTypeReference::InputObject(iotd),
                TypeDefinition::Interface(_)
                | TypeDefinition::Object(_)
                | TypeDefinition::Union(_) => {
                    panic!("Schema definition does not have unique type names");
                }
            })
    }

    pub(crate) fn base(&self) -> BaseInputTypeReference<'a, Self> {
        match self {
            Self::Base(base, _) => *base,
            Self::List(inner, _) => inner.base(),
        }
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::InputType
    for InputType<'a, S, W>
{
    type CustomScalarTypeDefinition = ScalarTypeDefinition<'a, S, W>;
    type EnumTypeDefinition = EnumTypeDefinition<'a, S, W>;
    type InputObjectTypeDefinition = InputObjectTypeDefinition<'a, S, W>;

    fn as_ref<
        'b,
        S2: SchemaDefinition<
            CustomScalarTypeDefinition = Self::CustomScalarTypeDefinition,
            InputObjectTypeDefinition = Self::InputObjectTypeDefinition,
            EnumTypeDefinition = Self::EnumTypeDefinition,
        >,
    >(
        &'b self,
        _: &'b S2,
    ) -> InputTypeReference<'b, Self> {
        match self {
            Self::Base(b, required) => InputTypeReference::Base(*b, *required),
            Self::List(inner, required) => InputTypeReference::List(inner, *required),
        }
    }

    fn as_shallow_ref(&self) -> definition::ShallowInputTypeReference<'_, Self> {
        match self {
            Self::Base(base, required) => {
                definition::ShallowInputTypeReference::Base(base.name(), *required)
            }
            Self::List(inner, required) => definition::ShallowInputTypeReference::List(
                std::ops::Deref::deref(inner),
                *required,
            ),
        }
    }
}
