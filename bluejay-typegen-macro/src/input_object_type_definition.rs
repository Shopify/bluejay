use crate::attributes::doc_string;
use crate::builtin_scalar::{builtin_scalar_type, scalar_is_reference};
use crate::names::{enum_variant_ident, field_ident, type_ident};
use crate::Config;
use bluejay_core::definition::{
    BaseInputType, BaseInputTypeReference, EnumTypeDefinition, InputObjectTypeDefinition,
    InputType, InputTypeReference, InputValueDefinition, ScalarTypeDefinition,
};
use bluejay_core::{AsIter, Directive};
use proc_macro2::{Ident, Span};
use std::collections::HashSet;
use syn::parse_quote;

pub(crate) fn generate_input_object_type_definition(
    iotd: &impl InputObjectTypeDefinition,
    config: &Config,
) -> Vec<syn::Item> {
    if matches!(iotd.directives(), Some(directives) if directives.iter().any(|directive| directive.name() == "oneOf"))
    {
        return generate_one_of_input_object_type_definition(iotd, config);
    }

    let description = iotd.description().map(doc_string);

    let ident = type_ident(iotd.name());

    let fields = &iotd
        .input_field_definitions()
        .iter()
        .map(|ivd| field_ident(ivd.name()))
        .collect::<Vec<Ident>>();

    let descriptions = &iotd
        .input_field_definitions()
        .iter()
        .map(|ivd| ivd.description().map(doc_string))
        .collect::<Vec<_>>();

    let types = &iotd
        .input_field_definitions()
        .iter()
        .map(|ivd| type_for_input_value_definition(iotd.name(), ivd, config))
        .collect::<Vec<syn::TypePath>>();

    let renamed = &iotd
        .input_field_definitions()
        .iter()
        .map(|ivd| syn::LitStr::new(ivd.name(), Span::call_site()))
        .collect::<Vec<syn::LitStr>>();

    let borrow = &iotd
        .input_field_definitions()
        .iter()
        .map(|ivd| {
            contains_reference_types(
                ivd.r#type().as_ref().base().as_ref(),
                config,
                &mut HashSet::new(),
            )
            .then(|| {
                parse_quote! { #[serde(borrow)] }
            })
        })
        .collect::<Vec<Option<syn::Attribute>>>();

    let lifetime = lifetime(iotd, config);

    vec![parse_quote! {
        #description
        #[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::cmp::Eq, ::std::fmt::Debug, ::bluejay_typegen::serde::Serialize)]
        #[serde(crate = "bluejay_typegen::serde")]
        pub struct #ident #lifetime {
            #(
                #descriptions
                #[serde(rename = #renamed)]
                #borrow
                pub #fields: #types,
            )* }
    }]
}

fn generate_one_of_input_object_type_definition(
    iotd: &impl InputObjectTypeDefinition,
    config: &Config,
) -> Vec<syn::Item> {
    let description = iotd.description().map(doc_string);

    let ident = type_ident(iotd.name());

    let variants = &iotd
        .input_field_definitions()
        .iter()
        .map(|ivd| enum_variant_ident(ivd.name()))
        .collect::<Vec<Ident>>();

    let descriptions = &iotd
        .input_field_definitions()
        .iter()
        .map(|ivd| ivd.description().map(doc_string))
        .collect::<Vec<_>>();

    let types = &iotd
        .input_field_definitions()
        .iter()
        .map(|ivd| {
            // kind of a hack
            let required_type = match ivd.r#type().as_ref() {
                InputTypeReference::Base(base, _) => InputTypeReference::Base(base, true),
                InputTypeReference::List(inner, _) => InputTypeReference::List(inner, true),
            };
            type_for_input_type(required_type, Some(iotd.name()), None, config)
        })
        .collect::<Vec<syn::TypePath>>();

    let serialized_as = &iotd
        .input_field_definitions()
        .iter()
        .map(|ivd| syn::LitStr::new(ivd.name(), Span::call_site()))
        .collect::<Vec<_>>();

    let borrow = &iotd
        .input_field_definitions()
        .iter()
        .map(|ivd| {
            contains_reference_types(
                ivd.r#type().as_ref().base().as_ref(),
                config,
                &mut HashSet::new(),
            )
            .then(|| {
                parse_quote! { #[serde(borrow)] }
            })
        })
        .collect::<Vec<Option<syn::Attribute>>>();

    let lifetime = lifetime(iotd, config);

    vec![parse_quote! {
        #description
        #[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::cmp::Eq, ::std::fmt::Debug, ::bluejay_typegen::serde::Serialize)]
        #[serde(crate = "bluejay_typegen::serde")]
        pub enum #ident #lifetime {
            #(
                #descriptions
                #[serde(rename = #serialized_as)]
                #variants(#borrow #types),
            )*
        }
    }]
}

fn type_for_input_value_definition(
    parent_type_name: &str,
    ivd: &impl InputValueDefinition,
    config: &Config,
) -> syn::TypePath {
    type_for_input_type(
        ivd.r#type().as_ref(),
        Some(parent_type_name),
        Some(ivd.default_value().is_some()),
        config,
    )
}

fn type_for_input_type<T: InputType>(
    ty: InputTypeReference<T>,
    parent_type_name: Option<&str>,
    has_default_value: Option<bool>,
    config: &Config,
) -> syn::TypePath {
    let required = has_default_value.map_or_else(
        || ty.is_required(),
        |has_default_value| !has_default_value && ty.is_required(),
    );
    match ty {
        InputTypeReference::Base(base, _) => {
            let mut inner = type_for_base_input_type(base.as_ref(), config);
            if let Some(parent_type_name) = parent_type_name {
                if contains_non_list_reference(parent_type_name, ty, &mut HashSet::new()) {
                    inner = parse_quote! { ::std::boxed::Box<#inner> };
                }
            }
            if required {
                inner
            } else {
                crate::types::option(inner)
            }
        }
        InputTypeReference::List(inner, _) => {
            let inner_ty =
                crate::types::vec(type_for_input_type(inner.as_ref(), None, None, config));
            if required {
                inner_ty
            } else {
                crate::types::option(inner_ty)
            }
        }
    }
}

fn type_for_base_input_type<T: BaseInputType>(
    base: BaseInputTypeReference<T>,
    config: &Config,
) -> syn::TypePath {
    match base {
        BaseInputTypeReference::BuiltinScalar(bstd) => builtin_scalar_type(bstd, config),
        BaseInputTypeReference::InputObject(iotd) => {
            let ident = type_ident(iotd.name());
            let lifetime = lifetime(iotd, config);
            parse_quote! { #ident #lifetime }
        }
        BaseInputTypeReference::Enum(etd) => {
            let ident = type_ident(etd.name());
            parse_quote! { #ident }
        }
        BaseInputTypeReference::CustomScalar(cstd) => {
            let ident = type_ident(cstd.name());
            let lifetime: Option<syn::Generics> = config
                .custom_scalar_borrows(cstd)
                .then(|| parse_quote! { <'a> });
            parse_quote! { #ident #lifetime }
        }
    }
}

fn input_object_contains_reference_types<'a>(
    iotd: &'a impl InputObjectTypeDefinition,
    config: &Config,
    visited: &mut HashSet<&'a str>,
) -> bool {
    iotd.input_field_definitions()
        .iter()
        .any(|ivd| contains_reference_types(ivd.r#type().as_ref().base().as_ref(), config, visited))
}

fn contains_reference_types<'a, T: BaseInputType>(
    ty: BaseInputTypeReference<'a, T>,
    config: &Config,
    visited: &mut HashSet<&'a str>,
) -> bool {
    if !config.borrow() || !visited.insert(ty.name()) {
        return false;
    }

    match ty {
        BaseInputTypeReference::BuiltinScalar(bstd) => scalar_is_reference(bstd),
        BaseInputTypeReference::CustomScalar(cstd) => config.custom_scalar_borrows(cstd),
        BaseInputTypeReference::Enum(_) => false,
        BaseInputTypeReference::InputObject(iotd) => {
            input_object_contains_reference_types(iotd, config, visited)
        }
    }
}

fn contains_non_list_reference<'a, T: InputType>(
    target: &str,
    ty: InputTypeReference<'a, T>,
    visited: &mut HashSet<&'a str>,
) -> bool {
    match ty {
        InputTypeReference::Base(base, _) if base.as_ref().name() == target => true,
        ty => match ty.base().as_ref() {
            BaseInputTypeReference::InputObject(iotd) => {
                if visited.insert(iotd.name()) {
                    iotd.input_field_definitions().iter().any(|ivd| {
                        contains_non_list_reference(target, ivd.r#type().as_ref(), visited)
                    })
                } else {
                    false
                }
            }
            _ => false,
        },
    }
}

fn lifetime(iotd: &impl InputObjectTypeDefinition, config: &Config) -> Option<syn::Generics> {
    (input_object_contains_reference_types(iotd, config, &mut HashSet::new()))
        .then(|| parse_quote! { <'a> })
}
