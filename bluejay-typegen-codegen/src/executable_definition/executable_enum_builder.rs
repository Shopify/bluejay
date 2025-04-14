use crate::executable_definition::{
    ExecutableEnum, ExecutableEnumVariantBuilder, ExecutableTypeBuilder,
};
use crate::{
    attributes::doc_string,
    names::{module_ident, type_ident},
    Config,
};
use bluejay_core::{definition::SchemaDefinition, AsIter};
use std::ops::Not;
use syn::parse_quote;

pub(crate) struct ExecutableEnumBuilder<'a, S: SchemaDefinition> {
    config: &'a Config<'a, S>,
    executable_enum: &'a ExecutableEnum<'a>,
    /// depth within the module for the executable document
    depth: usize,
}

impl<'a, S: SchemaDefinition> ExecutableEnumBuilder<'a, S> {
    pub(crate) fn build(
        executable_enum: &'a ExecutableEnum<'a>,
        config: &'a Config<'a, S>,
        depth: usize,
    ) -> Vec<syn::Item> {
        let instance = Self {
            config,
            executable_enum,
            depth,
        };

        let name_ident = instance.name_ident();
        let attributes = instance.attributes();
        let variants = instance.variants();
        let lifetime = instance.lifetime();

        let mut items = vec![parse_quote! {
            #(#attributes)*
            pub enum #name_ident #lifetime { #(#variants,)* }
        }];

        items.extend(instance.nested_module());

        items
    }

    fn name_ident(&self) -> syn::Ident {
        type_ident(self.executable_enum.parent_name)
    }

    fn attributes(&self) -> Vec<syn::Attribute> {
        let mut attributes = Vec::new();
        attributes.extend(self.executable_enum.description.map(doc_string));
        attributes.push(parse_quote! { #[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug, ::bluejay_typegen::serde::Deserialize)] });
        attributes.push(parse_quote! { #[serde(crate = "bluejay_typegen::serde")] });
        attributes.push(parse_quote! { #[serde(tag = "__typename")] });

        attributes
    }

    fn lifetime(&self) -> Option<syn::Generics> {
        self.executable_enum
            .borrows()
            .then(|| parse_quote! { <'a> })
    }

    fn variants(&self) -> Vec<syn::Variant> {
        self.executable_enum
            .variants
            .iter()
            .map(|variant| {
                ExecutableEnumVariantBuilder::build(
                    self.config,
                    variant,
                    self.depth,
                    self.executable_enum.parent_name,
                )
            })
            .chain(std::iter::once(
                ExecutableEnumVariantBuilder::<S>::build_other_variant(),
            ))
            .collect()
    }

    fn nested_module(&self) -> Option<syn::Item> {
        let variant_modules = self
            .executable_enum
            .variants
            .iter()
            .flat_map(|variant| {
                let modules_for_variant = variant
                    .fields
                    .iter()
                    .flat_map(|field| {
                        ExecutableTypeBuilder::build(
                            field.r#type.base(),
                            self.config,
                            self.depth + 2,
                        )
                    })
                    .collect::<Vec<syn::Item>>();

                modules_for_variant.is_empty().not().then(|| {
                    let module_ident = module_ident(variant.name);
                    parse_quote! {
                        pub mod #module_ident {
                            #(#modules_for_variant)*
                        }
                    }
                })
            })
            .collect::<Vec<syn::Item>>();

        variant_modules.is_empty().not().then(|| {
            let module_ident = module_ident(self.executable_enum.parent_name);
            parse_quote! {
                pub mod #module_ident {
                    #(#variant_modules)*
                }
            }
        })
    }
}
