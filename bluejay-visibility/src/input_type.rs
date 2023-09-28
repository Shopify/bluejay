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

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> InputType<'a, S, W> {
    pub fn new(inner: &'a S::InputType, cache: &'a Cache<'a, S, W>) -> Option<Self> {
        match inner.as_ref() {
            InputTypeReference::Base(b, required) => {
                Self::base(b, cache).map(|base| Self::Base(base, required))
            }
            InputTypeReference::List(inner, required) => {
                Self::new(inner, cache).map(|inner| Self::List(Box::new(inner), required))
            }
        }
    }

    fn base(
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
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::InputType
    for InputType<'a, S, W>
{
    type CustomScalarTypeDefinition = ScalarTypeDefinition<'a, S, W>;
    type EnumTypeDefinition = EnumTypeDefinition<'a, S, W>;
    type InputObjectTypeDefinition = InputObjectTypeDefinition<'a, S, W>;

    fn as_ref(&self) -> InputTypeReference<'_, Self> {
        match self {
            Self::Base(b, required) => InputTypeReference::Base(*b, *required),
            Self::List(inner, required) => InputTypeReference::List(inner, *required),
        }
    }
}
