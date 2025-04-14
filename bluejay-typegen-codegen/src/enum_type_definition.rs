use crate::attributes::doc_string;
use crate::names::{enum_variant_ident, type_ident};
use bluejay_core::{
    definition::{EnumTypeDefinition, EnumValueDefinition, SchemaDefinition},
    AsIter,
};
use proc_macro2::Span;
use syn::parse_quote;

pub(crate) struct EnumTypeDefinitionBuilder<'a, S: SchemaDefinition> {
    enum_type_definition: &'a S::EnumTypeDefinition,
}

impl<'a, S: SchemaDefinition> EnumTypeDefinitionBuilder<'a, S> {
    pub(crate) fn build(enum_type_definition: &'a S::EnumTypeDefinition) -> Vec<syn::Item> {
        let instance = Self {
            enum_type_definition,
        };

        let name_ident = instance.name_ident();
        let variant_description_attributes = instance.variant_description_attributes();
        let variant_serde_rename_attributes = instance.variant_serde_rename_attributes();
        let variant_idents = instance.variant_idents();
        let attributes = instance.attributes();
        let other_variant = instance.other_variant();

        vec![parse_quote! {
            #(#attributes)*
            pub enum #name_ident {
                #(
                    #variant_description_attributes
                    #variant_serde_rename_attributes
                    #variant_idents,
                )*
                #other_variant,
            }
        }]
    }

    fn name_ident(&self) -> syn::Ident {
        type_ident(self.enum_type_definition.name())
    }

    fn variant_idents(&self) -> Vec<syn::Ident> {
        self.enum_type_definition
            .enum_value_definitions()
            .iter()
            .map(|evd| enum_variant_ident(evd.name()))
            .collect()
    }

    fn variant_description_attributes(&self) -> Vec<Option<syn::Attribute>> {
        self.enum_type_definition
            .enum_value_definitions()
            .iter()
            .map(|evd| evd.description().map(doc_string))
            .collect()
    }

    fn variant_serialized_as(&self) -> Vec<syn::LitStr> {
        self.enum_type_definition
            .enum_value_definitions()
            .iter()
            .map(|evd| syn::LitStr::new(evd.name(), Span::call_site()))
            .collect()
    }

    fn variant_serde_rename_attributes(&self) -> Vec<syn::Attribute> {
        self.variant_serialized_as()
            .iter()
            .map(|serialized_as| parse_quote!(#[serde(rename = #serialized_as)]))
            .collect()
    }

    fn attributes(&self) -> Vec<syn::Attribute> {
        let mut attributes = Vec::new();
        attributes.extend(self.enum_type_definition.description().map(doc_string));
        attributes.push(parse_quote! { #[derive(::std::clone::Clone, ::std::marker::Copy, ::std::cmp::PartialEq, ::std::cmp::Eq, ::std::fmt::Debug, ::bluejay_typegen::serde::Serialize, ::bluejay_typegen::serde::Deserialize)] });
        attributes.push(parse_quote! { #[serde(crate = "bluejay_typegen::serde")] });

        attributes
    }

    fn other_variant(&self) -> syn::Variant {
        parse_quote! {
            #[serde(other)]
            Other
        }
    }
}
