use proc_macro2::Span;
use syn::parse_quote;

pub(crate) fn doc_string(s: &str) -> syn::Attribute {
    let description = syn::LitStr::new(s, Span::call_site());
    parse_quote! { #[doc = #description] }
}
