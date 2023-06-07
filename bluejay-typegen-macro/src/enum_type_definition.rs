use crate::attributes::doc_string;
use crate::names::{enum_variant_ident, type_ident};
use bluejay_core::definition::{EnumTypeDefinition, EnumValueDefinition};
use bluejay_core::AsIter;
use proc_macro2::Span;
use syn::parse_quote;

pub(crate) fn generate_enum_type_definition(etd: &impl EnumTypeDefinition) -> Vec<syn::Item> {
    let description = etd.description().map(doc_string);

    let ident = type_ident(etd.name());

    let variants = &etd
        .enum_value_definitions()
        .iter()
        .map(|evd| enum_variant_ident(evd.name()))
        .collect::<Vec<_>>();

    let descriptions = &etd
        .enum_value_definitions()
        .iter()
        .map(|evd| evd.description().map(doc_string))
        .collect::<Vec<_>>();

    let serialized_as = &etd
        .enum_value_definitions()
        .iter()
        .map(|evd| syn::LitStr::new(evd.name(), Span::call_site()))
        .collect::<Vec<syn::LitStr>>();

    vec![parse_quote! {
        #description
        #[derive(::std::clone::Clone, ::std::marker::Copy, ::std::cmp::PartialEq, ::std::cmp::Eq, ::std::fmt::Debug, ::bluejay_typegen::serde::Serialize, ::bluejay_typegen::serde::Deserialize)]
        #[serde(crate = "bluejay_typegen::serde")]
        pub enum #ident {
            #(
                #descriptions
                #[serde(rename = #serialized_as)]
                #variants
            ),*
        }
    }]
}
