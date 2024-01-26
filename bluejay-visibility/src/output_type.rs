use crate::{
    Cache, EnumTypeDefinition, InterfaceTypeDefinition, ObjectTypeDefinition, ScalarTypeDefinition,
    TypeDefinition, UnionTypeDefinition, Warden,
};
use bluejay_core::definition::{
    self, prelude::*, BaseOutputTypeReference, OutputTypeReference, SchemaDefinition,
    ShallowOutputTypeReference, TypeDefinitionReference,
};

pub enum OutputType<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S> + 'a> {
    Base(BaseOutputTypeReference<'a, Self>, bool),
    List(Box<Self>, bool),
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> OutputType<'a, S, W> {
    pub(crate) fn new(inner: &'a S::OutputType, cache: &'a Cache<'a, S, W>) -> Option<Self> {
        match inner.as_ref(cache.inner_schema_definition()) {
            OutputTypeReference::Base(b, required) => {
                Self::new_base(b, cache).map(|base| Self::Base(base, required))
            }
            OutputTypeReference::List(inner, required) => {
                Self::new(inner, cache).map(|inner| Self::List(Box::new(inner), required))
            }
        }
    }

    fn new_base(
        inner: BaseOutputTypeReference<'a, S::OutputType>,
        cache: &'a Cache<'a, S, W>,
    ) -> Option<BaseOutputTypeReference<'a, Self>> {
        let tdr = match inner {
            BaseOutputTypeReference::BuiltinScalar(bstd) => {
                TypeDefinitionReference::BuiltinScalar(bstd)
            }
            BaseOutputTypeReference::CustomScalar(cstd) => {
                TypeDefinitionReference::CustomScalar(cstd)
            }
            BaseOutputTypeReference::Enum(etd) => TypeDefinitionReference::Enum(etd),
            BaseOutputTypeReference::Interface(itd) => TypeDefinitionReference::Interface(itd),
            BaseOutputTypeReference::Object(otd) => TypeDefinitionReference::Object(otd),
            BaseOutputTypeReference::Union(utd) => TypeDefinitionReference::Union(utd),
        };

        cache
            .get_or_create_type_definition(tdr)
            .map(|type_definition| match type_definition {
                TypeDefinition::BuiltinScalar(bstd) => {
                    BaseOutputTypeReference::BuiltinScalar(*bstd)
                }
                TypeDefinition::CustomScalar(cstd) => BaseOutputTypeReference::CustomScalar(cstd),
                TypeDefinition::Enum(etd) => BaseOutputTypeReference::Enum(etd),
                TypeDefinition::Interface(itd) => BaseOutputTypeReference::Interface(itd),
                TypeDefinition::Object(otd) => BaseOutputTypeReference::Object(otd),
                TypeDefinition::Union(utd) => BaseOutputTypeReference::Union(utd),
                TypeDefinition::InputObject(_) => {
                    panic!("Schema definition does not have unique type names");
                }
            })
    }

    pub(crate) fn base(&self) -> BaseOutputTypeReference<'_, Self> {
        match self {
            Self::Base(b, _) => *b,
            Self::List(inner, _) => inner.base(),
        }
    }
}

impl<'a, S: SchemaDefinition + 'a, W: Warden<SchemaDefinition = S>> definition::OutputType
    for OutputType<'a, S, W>
{
    type ObjectTypeDefinition = ObjectTypeDefinition<'a, S, W>;
    type CustomScalarTypeDefinition = ScalarTypeDefinition<'a, S, W>;
    type InterfaceTypeDefinition = InterfaceTypeDefinition<'a, S, W>;
    type EnumTypeDefinition = EnumTypeDefinition<'a, S, W>;
    type UnionTypeDefinition = UnionTypeDefinition<'a, S, W>;

    fn as_ref<
        'b,
        S2: SchemaDefinition<
            CustomScalarTypeDefinition = Self::CustomScalarTypeDefinition,
            EnumTypeDefinition = Self::EnumTypeDefinition,
            ObjectTypeDefinition = Self::ObjectTypeDefinition,
            InterfaceTypeDefinition = Self::InterfaceTypeDefinition,
            UnionTypeDefinition = Self::UnionTypeDefinition,
        >,
    >(
        &'b self,
        _: &'b S2,
    ) -> OutputTypeReference<'b, Self> {
        match self {
            Self::Base(b, required) => OutputTypeReference::Base(*b, *required),
            Self::List(inner, required) => OutputTypeReference::List(inner.as_ref(), *required),
        }
    }

    fn as_shallow_ref(&self) -> ShallowOutputTypeReference<'_, Self> {
        match self {
            Self::Base(b, required) => ShallowOutputTypeReference::Base(b.name(), *required),
            Self::List(inner, required) => {
                ShallowOutputTypeReference::List(inner.as_ref(), *required)
            }
        }
    }
}
