use syn::parse_quote;

pub(crate) fn option(ty: syn::TypePath) -> syn::TypePath {
    parse_quote! { ::std::option::Option<#ty> }
}

pub(crate) fn vec(ty: syn::TypePath) -> syn::TypePath {
    parse_quote! { ::std::vec::Vec<#ty> }
}

pub(crate) fn string(borrows: bool) -> syn::TypePath {
    if borrows {
        parse_quote! { ::std::borrow::Cow<'a, str> }
    } else {
        parse_quote! { ::std::string::String }
    }
}
