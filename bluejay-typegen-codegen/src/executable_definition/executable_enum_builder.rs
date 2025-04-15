use crate::executable_definition::{
    ExecutableEnum, ExecutableEnumVariantBuilder, ExecutableStructBuilder,
};
use crate::{
    attributes::doc_string,
    names::{module_ident, type_ident},
    CodeGenerator,
};
use std::ops::Not;
use syn::parse_quote;

pub(crate) struct ExecutableEnumBuilder<'a, C: CodeGenerator> {
    executable_enum: &'a ExecutableEnum<'a>,
    code_generator: &'a C,
}

impl<'a, C: CodeGenerator> ExecutableEnumBuilder<'a, C> {
    pub(crate) fn build(
        executable_enum: &'a ExecutableEnum<'a>,
        code_generator: &'a C,
    ) -> Vec<syn::Item> {
        let instance = Self {
            executable_enum,
            code_generator,
        };

        let name_ident = instance.name_ident();
        let attributes = instance.attributes();
        let variants = instance.variants();
        let lifetime = instance.lifetime();

        let mut items = vec![parse_quote! {
            #(#attributes)*
            pub enum #name_ident #lifetime { #(#variants,)* }
        }];

        items.extend(
            instance
                .code_generator
                .additional_impls_for_executable_enum(instance.executable_enum)
                .into_iter()
                .map(Into::into),
        );

        items.extend(instance.nested_module());

        items
    }

    fn name_ident(&self) -> syn::Ident {
        type_ident(self.executable_enum.parent_name())
    }

    fn attributes(&self) -> Vec<syn::Attribute> {
        let mut attributes = Vec::new();
        attributes.extend(self.executable_enum.description().map(doc_string));
        attributes.extend(
            self.code_generator
                .attributes_for_executable_enum(self.executable_enum),
        );

        attributes
    }

    fn lifetime(&self) -> Option<syn::Generics> {
        self.executable_enum
            .borrows()
            .then(|| parse_quote! { <'a> })
    }

    fn variants(&self) -> Vec<syn::Variant> {
        self.executable_enum
            .variants()
            .iter()
            .map(|variant| {
                ExecutableEnumVariantBuilder::build(
                    variant,
                    self.code_generator,
                    self.executable_enum.parent_name(),
                )
            })
            .chain(std::iter::once(
                ExecutableEnumVariantBuilder::build_other_variant(self.code_generator),
            ))
            .collect()
    }

    fn nested_module(&self) -> Option<syn::Item> {
        let variant_structs = self
            .executable_enum
            .variants()
            .iter()
            .flat_map(|variant| ExecutableStructBuilder::build(variant, self.code_generator))
            .collect::<Vec<syn::Item>>();

        variant_structs.is_empty().not().then(|| {
            let module_ident = module_ident(self.executable_enum.parent_name());
            parse_quote! {
                pub mod #module_ident {
                    #(#variant_structs)*
                }
            }
        })
    }
}
