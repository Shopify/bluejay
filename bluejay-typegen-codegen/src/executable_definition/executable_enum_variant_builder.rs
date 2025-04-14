use crate::executable_definition::ExecutableStruct;
use crate::{
    attributes::doc_string,
    names::{module_ident, type_ident},
};
use proc_macro2::Span;
use syn::parse_quote;

pub(crate) struct ExecutableEnumVariantBuilder<'a> {
    executable_struct: &'a ExecutableStruct<'a>,
    /// name of the composite type that contains the field
    composite_type_name: &'a str,
}

impl<'a> ExecutableEnumVariantBuilder<'a> {
    pub(crate) fn build(
        executable_struct: &'a ExecutableStruct<'a>,
        composite_type_name: &'a str,
    ) -> syn::Variant {
        let instance = Self {
            executable_struct,
            composite_type_name,
        };

        let name_ident = instance.name_ident();
        let attributes = instance.attributes();
        let module_ident = instance.module_ident();
        let lifetime = instance.lifetime();

        parse_quote! {
            #(#attributes)*
            #name_ident(#module_ident :: #name_ident #lifetime)
        }
    }

    pub(crate) fn build_other_variant() -> syn::Variant {
        parse_quote! {
            #[serde(other)]
            Other
        }
    }

    pub(crate) fn name_ident(&self) -> syn::Ident {
        type_ident(self.executable_struct.parent_name)
    }

    pub(crate) fn module_ident(&self) -> syn::Ident {
        module_ident(self.composite_type_name)
    }

    pub(crate) fn serialized_as(&self) -> syn::LitStr {
        syn::LitStr::new(self.executable_struct.parent_name, Span::call_site())
    }

    fn attributes(&self) -> Vec<syn::Attribute> {
        let mut attributes = Vec::new();
        attributes.extend(self.executable_struct.description.map(doc_string));

        let serialized_as = self.serialized_as();
        attributes.push(parse_quote! { #[serde(rename = #serialized_as)] });

        if self.executable_struct.borrows() {
            attributes.push(parse_quote! { #[serde(borrow)] });
        }

        attributes
    }

    fn lifetime(&self) -> Option<syn::Generics> {
        self.executable_struct
            .borrows()
            .then(|| parse_quote! { <'a> })
    }
}
