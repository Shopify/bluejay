use bluejay_core::{
    definition::{InputValueDefinition, SchemaDefinition},
    Directive,
};
use std::marker::PhantomData;

pub trait Warden: Sized {
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

    fn is_directive_definition_visible(
        &self,
        directive_definition: &<Self::SchemaDefinition as SchemaDefinition>::DirectiveDefinition,
    ) -> bool;

    fn is_custom_scalar_type_definition_visible(
        &self,
        custom_scalar_type_definition: &<Self::SchemaDefinition as SchemaDefinition>::CustomScalarTypeDefinition,
    ) -> bool;

    fn is_enum_type_definition_visible(
        &self,
        enum_type_definition: &<Self::SchemaDefinition as SchemaDefinition>::EnumTypeDefinition,
    ) -> bool;

    fn is_input_object_type_definition_visible(
        &self,
        input_object_type_definition: &<Self::SchemaDefinition as SchemaDefinition>::InputObjectTypeDefinition,
    ) -> bool;

    fn is_interface_type_definition_visible(
        &self,
        interface_type_definition: &<Self::SchemaDefinition as SchemaDefinition>::InterfaceTypeDefinition,
    ) -> bool;

    fn is_object_type_definition_visible(
        &self,
        object_type_definition: &<Self::SchemaDefinition as SchemaDefinition>::ObjectTypeDefinition,
    ) -> bool;

    fn is_union_type_definition_visible(
        &self,
        union_type_definition: &<Self::SchemaDefinition as SchemaDefinition>::UnionTypeDefinition,
    ) -> bool;

    fn input_value_definition_default_value<'a>(
        &self,
        scoped_input_value_definition: &crate::InputValueDefinition<'a, Self::SchemaDefinition, Self>,
    ) -> Option<&'a <<Self::SchemaDefinition as SchemaDefinition>::InputValueDefinition as InputValueDefinition>::Value>{
        scoped_input_value_definition.inner().default_value()
    }

    fn directive_arguments<'a>(
        &self,
        scoped_directive: &crate::Directive<'a, Self::SchemaDefinition, Self>,
    ) -> Option<
        &'a <<Self::SchemaDefinition as SchemaDefinition>::Directive as Directive<true>>::Arguments,
    > {
        scoped_directive.inner().arguments()
    }
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

    fn is_directive_definition_visible(
        &self,
        _: &<Self::SchemaDefinition as SchemaDefinition>::DirectiveDefinition,
    ) -> bool {
        true
    }

    fn is_custom_scalar_type_definition_visible(
        &self,
        _: &<Self::SchemaDefinition as SchemaDefinition>::CustomScalarTypeDefinition,
    ) -> bool {
        true
    }

    fn is_enum_type_definition_visible(
        &self,
        _: &<Self::SchemaDefinition as SchemaDefinition>::EnumTypeDefinition,
    ) -> bool {
        true
    }

    fn is_input_object_type_definition_visible(
        &self,
        _: &<Self::SchemaDefinition as SchemaDefinition>::InputObjectTypeDefinition,
    ) -> bool {
        true
    }

    fn is_interface_type_definition_visible(
        &self,
        _: &<Self::SchemaDefinition as SchemaDefinition>::InterfaceTypeDefinition,
    ) -> bool {
        true
    }

    fn is_object_type_definition_visible(
        &self,
        _: &<Self::SchemaDefinition as SchemaDefinition>::ObjectTypeDefinition,
    ) -> bool {
        true
    }

    fn is_union_type_definition_visible(
        &self,
        _: &<Self::SchemaDefinition as SchemaDefinition>::UnionTypeDefinition,
    ) -> bool {
        true
    }
}
