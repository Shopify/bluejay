use crate::executable_definition::{ExecutableField, ExecutableType, WrappedExecutableType};
use crate::{
    attributes::doc_string,
    builtin_scalar::builtin_scalar_type,
    input::Codec,
    names::{field_ident, module_ident, type_ident},
    types, Config,
};
use bluejay_core::definition::SchemaDefinition;
use proc_macro2::Span;
use syn::parse_quote;

pub(crate) struct ExecutableFieldBuilder<'a, S: SchemaDefinition> {
    config: &'a Config<'a, S>,
    executable_field: &'a ExecutableField<'a>,
    /// depth within the module for the executable document
    depth: usize,
    /// name of the composite type that contains the field
    composite_type_name: &'a str,
    /// name of the enum variant that contains the field, if the field is part of an enum
    enum_variant_name: Option<&'a str>,
}

impl<'a, S: SchemaDefinition> ExecutableFieldBuilder<'a, S> {
    pub(crate) fn build(
        executable_field: &'a ExecutableField<'a>,
        config: &'a Config<'a, S>,
        depth: usize,
        composite_type_name: &'a str,
        enum_variant_name: Option<&'a str>,
    ) -> syn::Field {
        let instance = Self {
            config,
            executable_field,
            depth,
            composite_type_name,
            enum_variant_name,
        };

        let name_ident = instance.name_ident();
        let attributes = instance.attributes();
        let type_path = instance.type_path();
        let pub_token = instance.pub_token();

        parse_quote! {
            #(#attributes)*
            #pub_token #name_ident: #type_path
        }
    }

    pub(crate) fn new(
        executable_field: &'a ExecutableField<'a>,
        config: &'a Config<'a, S>,
        depth: usize,
        composite_type_name: &'a str,
        enum_variant_name: Option<&'a str>,
    ) -> Self {
        Self {
            config,
            executable_field,
            depth,
            composite_type_name,
            enum_variant_name,
        }
    }

    fn for_struct(&self) -> bool {
        self.enum_variant_name.is_none()
    }

    pub(crate) fn name_ident(&self) -> syn::Ident {
        field_ident(self.executable_field.graphql_name)
    }

    pub(crate) fn serialized_as(&self) -> syn::LitStr {
        syn::LitStr::new(self.executable_field.graphql_name, Span::call_site())
    }

    fn attributes(&self) -> Vec<syn::Attribute> {
        let mut attributes = Vec::new();
        attributes.extend(self.executable_field.description.map(doc_string));

        match self.config.codec() {
            Codec::Serde => {
                let serialized_as = self.serialized_as();
                attributes.push(parse_quote! { #[serde(rename = #serialized_as)] });
                if self.executable_field.r#type.base().borrows() {
                    attributes.push(parse_quote! { #[serde(borrow)] });
                }
            }
            Codec::Miniserde => {}
        }

        attributes
    }

    pub(crate) fn type_path(&self) -> syn::TypePath {
        self.compute_type_path(&self.executable_field.r#type)
    }

    fn compute_type_path(&self, r#type: &WrappedExecutableType<'a>) -> syn::TypePath {
        match r#type {
            WrappedExecutableType::Base(base) => match base {
                ExecutableType::Leaf {
                    path_segments,
                    borrows,
                } => {
                    let prefix = self.prefix_for_schema_definition_module();
                    let lifetime: Option<syn::Generics> = borrows.then(|| parse_quote! { <'a> });
                    parse_quote! { #(#prefix::)* #(#path_segments)::* #lifetime }
                }
                ExecutableType::BuiltinScalar { bstd, .. } => {
                    builtin_scalar_type(*bstd, self.config)
                }
                ExecutableType::FragmentDefinitionReference { name, borrows } => {
                    let prefix = self.prefix_for_executable_document_module();
                    let type_ident = type_ident(name);
                    let lifetime: Option<syn::Generics> = borrows.then(|| parse_quote! { <'a> });
                    parse_quote! { #(#prefix::)* #type_ident #lifetime }
                }
                ExecutableType::Struct(es) => {
                    let prefix = std::iter::once(module_ident(self.composite_type_name))
                        .chain(self.enum_variant_name.map(module_ident));
                    let type_ident = type_ident(es.parent_name);
                    let lifetime: Option<syn::Generics> =
                        es.borrows().then(|| parse_quote! { <'a> });
                    parse_quote! { #(#prefix::)* #type_ident #lifetime }
                }
                ExecutableType::Enum(ee) => {
                    let prefix = std::iter::once(module_ident(self.composite_type_name))
                        .chain(self.enum_variant_name.map(module_ident));
                    let type_ident = type_ident(ee.parent_name);
                    let lifetime: Option<syn::Generics> =
                        ee.borrows().then(|| parse_quote! { <'a> });
                    parse_quote! { #(#prefix::)* #type_ident #lifetime }
                }
            },
            WrappedExecutableType::Optional(inner) => types::option(self.compute_type_path(inner)),
            WrappedExecutableType::Vec(inner) => types::vec(self.compute_type_path(inner)),
        }
    }

    fn prefix_for_schema_definition_module(&self) -> impl Iterator<Item = syn::Token![super]> {
        // root is one level higher than the executable/query module
        std::iter::repeat(Default::default()).take(self.depth + 1)
    }

    fn prefix_for_executable_document_module(&self) -> impl Iterator<Item = syn::Token![super]> {
        std::iter::repeat(Default::default()).take(self.depth)
    }

    fn pub_token(&self) -> Option<syn::Token![pub]> {
        self.for_struct().then(Default::default)
    }
}
