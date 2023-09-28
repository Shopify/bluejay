use crate::attributes::doc_string;
use crate::executable_definition::{generate_union_type_definition, Context};
use crate::names::{field_ident, module_ident, type_ident};
use bluejay_core::definition::{
    BaseOutputTypeReference, FieldDefinition, FieldsDefinition, InterfaceTypeDefinition,
    ObjectTypeDefinition, OutputType,
};
use bluejay_core::executable::{Field, Selection, SelectionReference, SelectionSet};
use proc_macro2::Span;
use std::ops::Not;
use syn::parse_quote;

pub(crate) fn generate_object_type_definition(
    object_type_definition: &impl ObjectTypeDefinition,
    selection_set: &impl SelectionSet,
    context: Context,
) -> Vec<syn::Item> {
    generate_type_for_selections(
        object_type_definition.description(),
        object_type_definition.fields_definition(),
        selection_set,
        context,
    )
}

pub(crate) fn generate_interface_type_definition(
    interface_type_definition: &impl InterfaceTypeDefinition,
    selection_set: &impl SelectionSet,
    context: Context,
) -> Vec<syn::Item> {
    generate_type_for_selections(
        interface_type_definition.description(),
        interface_type_definition.fields_definition(),
        selection_set,
        context,
    )
}

fn generate_type_for_selections(
    description: Option<&str>,
    fields_definition: &impl FieldsDefinition,
    selection_set: &impl SelectionSet,
    context: Context,
) -> Vec<syn::Item> {
    let ident = type_ident(context.name());
    let description = description.map(doc_string);

    let fields_and_definitions = fields_and_definitions(selection_set, fields_definition);

    // when fragment spread is used
    if fields_and_definitions.is_empty() {
        return Vec::new();
    }

    let named_fields = named_fields(&fields_and_definitions, &context, true);

    let lifetime = context.lifetime(&fields_and_definitions);

    let mut items = vec![parse_quote! {
        #description
        #[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::cmp::Eq, ::std::fmt::Debug, ::bluejay_typegen::serde::Deserialize)]
        #[serde(crate = "bluejay_typegen::serde")]
        pub struct #ident #lifetime #named_fields
    }];

    items.extend(nested_module(&fields_and_definitions, &context));

    items
}

pub(crate) fn fields_and_definitions<'a, S: SelectionSet, F: FieldsDefinition>(
    selection_set: &'a S,
    fields_definition: &'a F,
) -> Vec<(
    &'a <S::Selection as Selection>::Field,
    &'a F::FieldDefinition,
)> {
    selection_set
        .iter()
        .filter_map(|selection| match selection.as_ref() {
            SelectionReference::Field(f) => Some((f, fields_definition.get(f.name()).unwrap())),
            _ => None,
        })
        .collect()
}

pub(crate) fn named_fields(
    fields_and_definitions: &[(&impl Field, &impl FieldDefinition)],
    context: &Context,
    for_struct: bool,
) -> syn::FieldsNamed {
    let fields = &fields_and_definitions
        .iter()
        .map(|(f, _)| field_ident(f.response_name()))
        .collect::<Vec<_>>();

    let rename = &fields_and_definitions
        .iter()
        .map(|(f, _)| syn::LitStr::new(f.response_name(), Span::call_site()))
        .collect::<Vec<syn::LitStr>>();

    let descriptions = &fields_and_definitions
        .iter()
        .map(|(_, fd)| fd.description().map(doc_string))
        .collect::<Vec<_>>();

    let types = &fields_and_definitions
        .iter()
        .map(|(f, fd)| context.type_for_output_type(fd.r#type().as_ref(), *f))
        .collect::<Vec<_>>();

    let borrow = &fields_and_definitions
        .iter()
        .map(|(f, fd)| {
            context
                .contains_reference_types(*f, *fd, &mut Default::default())
                .then(|| parse_quote! { #[serde(borrow)] })
        })
        .collect::<Vec<Option<syn::Attribute>>>();

    let pub_token: Option<syn::Token![pub]> = for_struct.then(|| parse_quote! { pub });

    parse_quote! {
        {
            #(
                #descriptions
                #[serde(rename = #rename)]
                #borrow
                #pub_token #fields: #types,
            )*
        }
    }
}

pub(crate) fn nested_module(
    fields_and_definitions: &[(&impl Field, &impl FieldDefinition)],
    context: &Context,
) -> Option<syn::Item> {
    let nested = fields_and_definitions
        .iter()
        .flat_map(
            |(field, field_definition)| match field_definition.r#type().as_ref().base() {
                BaseOutputTypeReference::Object(otd) => generate_object_type_definition(
                    otd,
                    field.selection_set().unwrap(),
                    context.dive(field.response_name()),
                ),
                BaseOutputTypeReference::Union(utd) => generate_union_type_definition(
                    utd,
                    field.selection_set().unwrap(),
                    context.dive(field.response_name()),
                ),
                BaseOutputTypeReference::Interface(itd) => generate_interface_type_definition(
                    itd,
                    field.selection_set().unwrap(),
                    context.dive(field.response_name()),
                ),
                _ => Vec::new(),
            },
        )
        .collect::<Vec<syn::Item>>();

    nested.is_empty().not().then(|| {
        let module_ident = module_ident(context.enum_variant().unwrap_or(context.name()));
        parse_quote! {
            pub mod #module_ident {
                #(#nested)*
            }
        }
    })
}
