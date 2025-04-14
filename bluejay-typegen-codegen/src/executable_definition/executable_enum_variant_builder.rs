use crate::executable_definition::{ExecutableEnumVariant, ExecutableFieldBuilder};
use crate::{attributes::doc_string, names::type_ident, Config};
use bluejay_core::{definition::SchemaDefinition, AsIter};
use proc_macro2::Span;
use syn::parse_quote;

pub(crate) struct ExecutableEnumVariantBuilder<'a, S: SchemaDefinition> {
    config: &'a Config<'a, S>,
    executable_enum_variant: &'a ExecutableEnumVariant<'a>,
    /// depth within the module for the executable document
    depth: usize,
    /// name of the composite type that contains the field
    composite_type_name: &'a str,
}

impl<'a, S: SchemaDefinition> ExecutableEnumVariantBuilder<'a, S> {
    pub(crate) fn build(
        config: &'a Config<'a, S>,
        executable_enum_variant: &'a ExecutableEnumVariant<'a>,
        depth: usize,
        composite_type_name: &'a str,
    ) -> syn::Variant {
        let instance = Self {
            config,
            executable_enum_variant,
            depth,
            composite_type_name,
        };

        let name_ident = instance.name_ident();
        let attributes = instance.attributes();
        let fields = instance.fields();

        parse_quote! {
            #(#attributes)*
            #name_ident #fields
        }
    }

    pub(crate) fn build_other_variant() -> syn::Variant {
        parse_quote! {
            #[serde(other)]
            Other
        }
    }

    pub(crate) fn name_ident(&self) -> syn::Ident {
        type_ident(self.executable_enum_variant.name)
    }

    pub(crate) fn serialized_as(&self) -> syn::LitStr {
        syn::LitStr::new(self.executable_enum_variant.name, Span::call_site())
    }

    fn attributes(&self) -> Vec<syn::Attribute> {
        let mut attributes = Vec::new();
        attributes.extend(self.executable_enum_variant.description.map(doc_string));

        let serialized_as = self.serialized_as();
        attributes.push(parse_quote! { #[serde(rename = #serialized_as)] });

        attributes
    }

    fn fields(&self) -> syn::FieldsNamed {
        let fields = self
            .executable_enum_variant
            .fields
            .iter()
            .map(|field| {
                ExecutableFieldBuilder::build(
                    field,
                    self.config,
                    self.depth,
                    self.composite_type_name,
                    Some(self.executable_enum_variant.name),
                )
            })
            .collect::<Vec<syn::Field>>();
        parse_quote! { { #(#fields,)* } }
    }
}
