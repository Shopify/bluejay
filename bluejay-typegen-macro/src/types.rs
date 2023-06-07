use syn::parse_quote;

pub(crate) fn option(ty: syn::TypePath) -> syn::TypePath {
    parse_quote! { ::std::option::Option<#ty> }
}

pub(crate) fn vec(ty: syn::TypePath) -> syn::TypePath {
    parse_quote! { ::std::vec::Vec<#ty> }
}
