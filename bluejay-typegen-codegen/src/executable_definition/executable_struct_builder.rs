use crate::executable_definition::{
    ExecutableFieldBuilder, ExecutableStruct, ExecutableTypeBuilder,
};
use crate::{
    attributes::doc_string,
    names::{module_ident, type_ident},
    Config,
};
use bluejay_core::definition::SchemaDefinition;
use std::ops::Not;
use syn::parse_quote;

pub(crate) struct ExecutableStructBuilder<'a, S: SchemaDefinition> {
    config: &'a Config<'a, S>,
    executable_struct: &'a ExecutableStruct<'a>,
    /// depth within the module for the executable document
    depth: usize,
}

impl<'a, S: SchemaDefinition> ExecutableStructBuilder<'a, S> {
    pub(crate) fn build(
        executable_struct: &'a ExecutableStruct<'a>,
        config: &'a Config<'a, S>,
        depth: usize,
    ) -> Vec<syn::Item> {
        let instance = Self {
            config,
            executable_struct,
            depth,
        };

        let name_ident = instance.name_ident();
        let attributes = instance.attributes();
        let fields = instance.fields();
        let lifetime = instance.lifetime();

        let mut items = vec![parse_quote! {
            #(#attributes)*
            pub struct #name_ident #lifetime #fields
        }];

        items.extend(instance.nested_module());

        items
    }

    fn name_ident(&self) -> syn::Ident {
        type_ident(self.executable_struct.parent_name)
    }

    fn attributes(&self) -> Vec<syn::Attribute> {
        let mut attributes = Vec::new();
        attributes.extend(self.executable_struct.description.map(doc_string));
        attributes.push(parse_quote! { #[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug, ::bluejay_typegen::serde::Deserialize)] });
        attributes.push(parse_quote! { #[serde(crate = "bluejay_typegen::serde")] });

        attributes
    }

    fn fields(&self) -> syn::FieldsNamed {
        let fields = self
            .executable_struct
            .fields
            .iter()
            .map(|field| {
                ExecutableFieldBuilder::build(
                    field,
                    self.config,
                    self.depth,
                    self.executable_struct.parent_name,
                )
            })
            .collect::<Vec<syn::Field>>();
        parse_quote! { { #(#fields,)* } }
    }

    fn nested_module(&self) -> Option<syn::Item> {
        let nested = self
            .executable_struct
            .fields
            .iter()
            .flat_map(|field| {
                ExecutableTypeBuilder::build(field.r#type.base(), self.config, self.depth + 1)
            })
            .collect::<Vec<syn::Item>>();

        nested.is_empty().not().then(|| {
            let module_ident = module_ident(self.executable_struct.parent_name);
            parse_quote! {
                pub mod #module_ident {
                    #(#nested)*
                }
            }
        })
    }

    fn lifetime(&self) -> Option<syn::Generics> {
        self.executable_struct
            .borrows()
            .then(|| parse_quote! { <'a> })
    }
}
