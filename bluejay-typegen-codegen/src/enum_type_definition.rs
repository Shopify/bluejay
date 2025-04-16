use crate::{
    attributes::doc_string,
    names::{enum_variant_ident, type_ident},
    CodeGenerator,
};
use bluejay_core::{
    definition::{EnumTypeDefinition, EnumValueDefinition, SchemaDefinition},
    AsIter,
};
use syn::parse_quote;

pub(crate) struct EnumTypeDefinitionBuilder<'a, S: SchemaDefinition, C: CodeGenerator> {
    enum_type_definition: &'a S::EnumTypeDefinition,
    code_generator: &'a C,
}

impl<'a, S: SchemaDefinition, C: CodeGenerator> EnumTypeDefinitionBuilder<'a, S, C> {
    pub(crate) fn build(
        enum_type_definition: &'a S::EnumTypeDefinition,
        code_generator: &'a C,
    ) -> Vec<syn::Item> {
        let instance = Self {
            enum_type_definition,
            code_generator,
        };

        let name_ident = instance.name_ident();
        let attributes = instance.attributes();
        let other_variant = instance.other_variant();

        let variants: Vec<syn::Variant> = instance
            .enum_type_definition
            .enum_value_definitions()
            .iter()
            .map(|evd| {
                let variant_ident = enum_variant_ident(evd.name());
                let description_attribute = evd.description().map(doc_string);
                let other_attributes = instance.code_generator.attributes_for_enum_variant(evd);

                parse_quote! {
                    #description_attribute
                    #(#other_attributes)*
                    #variant_ident
                }
            })
            .collect();

        let mut items = vec![parse_quote! {
            #(#attributes)*
            pub enum #name_ident {
                #(
                    #variants,
                )*
                #other_variant,
            }
        }];

        items.extend(
            instance
                .code_generator
                .additional_impls_for_enum(instance.enum_type_definition)
                .into_iter()
                .map(Into::into),
        );

        items
    }

    fn name_ident(&self) -> syn::Ident {
        type_ident(self.enum_type_definition.name())
    }

    fn attributes(&self) -> Vec<syn::Attribute> {
        self.enum_type_definition
            .description()
            .map(doc_string)
            .into_iter()
            .chain(
                self.code_generator
                    .attributes_for_enum(self.enum_type_definition),
            )
            .collect()
    }

    fn other_variant(&self) -> syn::Variant {
        let attributes = self.code_generator.attributes_for_enum_variant_other();
        parse_quote! {
            #(#attributes)*
            Other
        }
    }
}
