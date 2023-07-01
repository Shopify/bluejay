use bluejay_core::definition::SchemaDefinition;
use std::marker::PhantomData;

pub trait Warden {
    type SchemaDefinition: SchemaDefinition;

    fn is_field_definition_visible(
        &self,
        field_definition: &<Self::SchemaDefinition as SchemaDefinition>::FieldDefinition,
    ) -> bool;

    fn is_input_value_definition_visible(
        &self,
        input_value_definition: &<Self::SchemaDefinition as SchemaDefinition>::InputValueDefinition,
    ) -> bool;

    fn is_enum_value_definition_visible(
        &self,
        enum_value_definition: &<Self::SchemaDefinition as SchemaDefinition>::EnumValueDefinition,
    ) -> bool;

    fn is_union_member_type_visible(
        &self,
        union_member_type: &<Self::SchemaDefinition as SchemaDefinition>::UnionMemberType,
    ) -> bool;

    fn is_interface_implementation_visible(
        &self,
        interface_implementation: &<Self::SchemaDefinition as SchemaDefinition>::InterfaceImplementation,
    ) -> bool;
}

pub struct NullWarden<S: SchemaDefinition>(PhantomData<S>);

impl<S: SchemaDefinition> Default for NullWarden<S> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<S: SchemaDefinition> Warden for NullWarden<S> {
    type SchemaDefinition = S;

    fn is_field_definition_visible(
        &self,
        _: &<Self::SchemaDefinition as SchemaDefinition>::FieldDefinition,
    ) -> bool {
        true
    }

    fn is_input_value_definition_visible(
        &self,
        _: &<Self::SchemaDefinition as SchemaDefinition>::InputValueDefinition,
    ) -> bool {
        true
    }

    fn is_enum_value_definition_visible(
        &self,
        _: &<Self::SchemaDefinition as SchemaDefinition>::EnumValueDefinition,
    ) -> bool {
        true
    }

    fn is_union_member_type_visible(
        &self,
        _: &<Self::SchemaDefinition as SchemaDefinition>::UnionMemberType,
    ) -> bool {
        true
    }

    fn is_interface_implementation_visible(
        &self,
        _: &<Self::SchemaDefinition as SchemaDefinition>::InterfaceImplementation,
    ) -> bool {
        true
    }
}
