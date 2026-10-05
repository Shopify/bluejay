//! Rust types for GraphQL input types, shared by input objects and the structs for operation variables.

use crate::builtin_scalar::{builtin_scalar_type, scalar_is_reference};
use crate::names::type_ident;
use crate::{types, CodeGenerator, Config};
use bluejay_core::definition::{prelude::*, BaseInputTypeReference, SchemaDefinition};
use bluejay_core::AsIter;
use std::collections::HashSet;
use syn::parse_quote;

/// The type for `base`, used from a module `schema_module_depth` levels below the schema definition module, which is
/// where enums, input objects, and custom scalar type aliases are defined.
pub(crate) fn base_input_type<'a, S: SchemaDefinition, C: CodeGenerator>(
    config: &Config<'a, S, C>,
    base: BaseInputTypeReference<'a, S>,
    schema_module_depth: usize,
) -> syn::Type {
    let prefix = std::iter::repeat_n(<syn::Token![super]>::default(), schema_module_depth);
    match base {
        BaseInputTypeReference::BuiltinScalar(bstd) => builtin_scalar_type(bstd, config.borrow()),
        BaseInputTypeReference::InputObject(iotd) => {
            let ident = type_ident(iotd.name());
            let lifetime = input_object_lifetime(config, iotd);
            parse_quote! { #(#prefix::)* #ident #lifetime }
        }
        BaseInputTypeReference::Enum(etd) => {
            if config.enum_as_str(etd) {
                types::string(config.borrow())
            } else {
                let ident = type_ident(etd.name());
                parse_quote! { #(#prefix::)* #ident }
            }
        }
        BaseInputTypeReference::CustomScalar(cstd) => {
            let ident = type_ident(cstd.name());
            let lifetime: Option<syn::Generics> = config
                .custom_scalar_borrows(cstd)
                .then(|| parse_quote! { <'a> });
            parse_quote! { #(#prefix::)* #ident #lifetime }
        }
    }
}

pub(crate) fn input_object_lifetime<'a, S: SchemaDefinition, C: CodeGenerator>(
    config: &Config<'a, S, C>,
    iotd: &'a S::InputObjectTypeDefinition,
) -> Option<syn::Generics> {
    input_object_borrows(config, iotd, &mut HashSet::new()).then(|| parse_quote! { <'a> })
}

/// Whether the type for `base` borrows, skipping types already in `visited`.
pub(crate) fn base_input_type_borrows<'a, S: SchemaDefinition, C: CodeGenerator>(
    config: &Config<'a, S, C>,
    base: BaseInputTypeReference<'a, S>,
    visited: &mut HashSet<&'a str>,
) -> bool {
    if !config.borrow() || !visited.insert(base.name()) {
        return false;
    }

    match base {
        BaseInputTypeReference::BuiltinScalar(bstd) => scalar_is_reference(bstd),
        BaseInputTypeReference::CustomScalar(cstd) => config.custom_scalar_borrows(cstd),
        BaseInputTypeReference::Enum(etd) => config.enum_as_str(etd),
        BaseInputTypeReference::InputObject(iotd) => input_object_borrows(config, iotd, visited),
    }
}

/// Whether the type for `ivd`, a field of `iotd`, borrows, skipping types already in `visited`.
pub(crate) fn input_field_borrows<'a, S: SchemaDefinition, C: CodeGenerator>(
    config: &Config<'a, S, C>,
    iotd: &'a S::InputObjectTypeDefinition,
    ivd: &'a S::InputValueDefinition,
    visited: &mut HashSet<&'a str>,
) -> bool {
    config.custom_scalar_override(iotd, ivd).map_or_else(
        || {
            base_input_type_borrows(
                config,
                ivd.r#type().base(config.schema_definition()),
                visited,
            )
        },
        |custom_scalar_override| custom_scalar_override.borrows,
    )
}

fn input_object_borrows<'a, S: SchemaDefinition, C: CodeGenerator>(
    config: &Config<'a, S, C>,
    iotd: &'a S::InputObjectTypeDefinition,
    visited: &mut HashSet<&'a str>,
) -> bool {
    iotd.input_field_definitions()
        .iter()
        .any(|ivd| input_field_borrows(config, iotd, ivd, visited))
}
