use bluejay_core::{
    definition::{
        prelude::*, InputValueDefinition, ScalarTypeDefinition, SchemaDefinition,
        TypeDefinitionReference,
    },
    Directive,
};
use std::marker::PhantomData;

pub trait Warden: Sized {
    type SchemaDefinition: SchemaDefinition;
    type Id<'a>: Eq;
    type TypeDefinitionsForName<'a>: Iterator<
            Item = TypeDefinitionReference<
                'a,
                <Self::SchemaDefinition as SchemaDefinition>::TypeDefinition,
            >,
        > + 'a
    where
        Self: 'a;

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

    // Each of the following methods is a sign that we should have more specific traits than the `bluejay-core::definition`
    // ones. The more we add, the greater the argument that we should create dedicated traits instead of using the core ones.

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

    fn custom_scalar_definition_coerce_input<const CONST: bool>(
        &self,
        custom_scalar_type_definition: &<Self::SchemaDefinition as SchemaDefinition>::CustomScalarTypeDefinition,
        value: &impl bluejay_core::Value<CONST>,
    ) -> Result<(), std::borrow::Cow<'static, str>> {
        custom_scalar_type_definition.coerce_input(value)
    }

    fn type_definitions_for_name<'a>(
        &self,
        schema_definition: &'a Self::SchemaDefinition,
        type_name: &str,
    ) -> Self::TypeDefinitionsForName<'a>;

    fn object_type_definition_id<'a>(
        &self,
        object_type_definition: &'a <Self::SchemaDefinition as SchemaDefinition>::ObjectTypeDefinition,
    ) -> Self::Id<'a>;

    fn scalar_type_definition_id<'a>(
        &self,
        scalar_type_definition: &'a <Self::SchemaDefinition as SchemaDefinition>::CustomScalarTypeDefinition,
    ) -> Self::Id<'a>;

    fn enum_type_definition_id<'a>(
        &self,
        enum_type_definition: &'a <Self::SchemaDefinition as SchemaDefinition>::EnumTypeDefinition,
    ) -> Self::Id<'a>;

    fn input_object_type_definition_id<'a>(
        &self,
        input_object_type_definition: &'a <Self::SchemaDefinition as SchemaDefinition>::InputObjectTypeDefinition,
    ) -> Self::Id<'a>;

    fn interface_type_definition_id<'a>(
        &self,
        interface_type_definition: &'a <Self::SchemaDefinition as SchemaDefinition>::InterfaceTypeDefinition,
    ) -> Self::Id<'a>;

    fn union_type_definition_id<'a>(
        &self,
        union_type_definition: &'a <Self::SchemaDefinition as SchemaDefinition>::UnionTypeDefinition,
    ) -> Self::Id<'a>;
}

pub struct NullWarden<S: SchemaDefinition>(PhantomData<S>);

impl<S: SchemaDefinition> Default for NullWarden<S> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<S: SchemaDefinition> Warden for NullWarden<S> {
    type SchemaDefinition = S;
    type Id<'a> = &'a str;
    type TypeDefinitionsForName<'a> =
        std::option::IntoIter<TypeDefinitionReference<'a, S::TypeDefinition>> where Self: 'a;

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

    fn object_type_definition_id<'a>(
        &self,
        object_type_definition: &'a <Self::SchemaDefinition as SchemaDefinition>::ObjectTypeDefinition,
    ) -> Self::Id<'a> {
        object_type_definition.name()
    }

    fn scalar_type_definition_id<'a>(
        &self,
        scalar_type_definition: &'a <Self::SchemaDefinition as SchemaDefinition>::CustomScalarTypeDefinition,
    ) -> Self::Id<'a> {
        scalar_type_definition.name()
    }

    fn enum_type_definition_id<'a>(
        &self,
        enum_type_definition: &'a <Self::SchemaDefinition as SchemaDefinition>::EnumTypeDefinition,
    ) -> Self::Id<'a> {
        enum_type_definition.name()
    }

    fn input_object_type_definition_id<'a>(
        &self,
        input_object_type_definition: &'a <Self::SchemaDefinition as SchemaDefinition>::InputObjectTypeDefinition,
    ) -> Self::Id<'a> {
        input_object_type_definition.name()
    }

    fn interface_type_definition_id<'a>(
        &self,
        interface_type_definition: &'a <Self::SchemaDefinition as SchemaDefinition>::InterfaceTypeDefinition,
    ) -> Self::Id<'a> {
        interface_type_definition.name()
    }

    fn union_type_definition_id<'a>(
        &self,
        union_type_definition: &'a <Self::SchemaDefinition as SchemaDefinition>::UnionTypeDefinition,
    ) -> Self::Id<'a> {
        union_type_definition.name()
    }

    fn type_definitions_for_name<'a>(
        &self,
        schema_definition: &'a Self::SchemaDefinition,
        type_name: &str,
    ) -> Self::TypeDefinitionsForName<'a> {
        schema_definition.get_type_definition(type_name).into_iter()
    }
}
