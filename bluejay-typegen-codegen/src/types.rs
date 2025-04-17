use syn::parse_quote;

pub(crate) fn option(ty: syn::Type) -> syn::Type {
    parse_quote! { ::std::option::Option<#ty> }
}

pub(crate) fn vec(ty: syn::Type) -> syn::Type {
    parse_quote! { ::std::vec::Vec<#ty> }
}

pub(crate) fn string(borrows: bool) -> syn::Type {
    if borrows {
        parse_quote! { ::std::borrow::Cow<'a, str> }
    } else {
        parse_quote! { ::std::string::String }
    }
}
