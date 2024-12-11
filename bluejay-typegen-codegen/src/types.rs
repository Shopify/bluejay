use crate::Config;
use bluejay_core::definition::SchemaDefinition;
use syn::parse_quote;

pub(crate) fn option(ty: syn::TypePath) -> syn::TypePath {
    parse_quote! { ::std::option::Option<#ty> }
}

pub(crate) fn vec(ty: syn::TypePath) -> syn::TypePath {
    parse_quote! { ::std::vec::Vec<#ty> }
}

pub(crate) fn string<S: SchemaDefinition>(config: &Config<S>) -> syn::TypePath {
    if config.borrow() {
        parse_quote! { ::std::borrow::Cow<'a, str> }
    } else {
        parse_quote! { ::std::string::String }
    }
}
