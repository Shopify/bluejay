use crate::executable_definition::{
    ExecutableFieldBuilder, ExecutableStruct, ExecutableTypeBuilder,
};
use crate::{
    attributes::doc_string,
    names::{module_ident, type_ident},
    types::option,
    Codec, Config,
};
use bluejay_core::definition::SchemaDefinition;
use quote::format_ident;
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
        items.extend(instance.miniserde_deserialize_impl());

        items
    }

    fn name_ident(&self) -> syn::Ident {
        type_ident(self.executable_struct.parent_name)
    }

    fn attributes(&self) -> Vec<syn::Attribute> {
        let mut attributes = Vec::new();
        attributes.extend(self.executable_struct.description.map(doc_string));
        attributes.push(parse_quote! { #[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)] });
        let serde_path = self.config.serde_path();
        let serde_path_lit_str = self.config.serde_path_lit_str();

        match self.config.codec() {
            Codec::Serde => {
                attributes.push(parse_quote! { #[derive(#serde_path ::Deserialize)] });
                attributes.push(parse_quote! { #[serde(crate = #serde_path_lit_str)] });
            }
            Codec::Miniserde => {}
        }

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
                    None,
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

    fn miniserde_deserialize_impl(&self) -> Option<syn::Item> {
        (self.config.codec() == Codec::Miniserde).then(|| {
            let name_ident = self.name_ident();
            let builder_ident = format_ident!("{}Builder", name_ident);

            let field_builders = self.field_builders();

            let builder_fields = field_builders.iter().map(|field_builder| {
                let name_ident = field_builder.name_ident();
                let type_path = option(field_builder.type_path());
                parse_quote! { #name_ident: #type_path }
            }).collect::<Vec<syn::Field>>();

            let field_idents = field_builders.iter().map(ExecutableFieldBuilder::name_ident).collect::<Vec<syn::Ident>>();

            let field_serialized_as = field_builders.iter().map(ExecutableFieldBuilder::serialized_as).collect::<Vec<syn::LitStr>>();

            let miniserde_path = self.config.miniserde_path();

            parse_quote! {
                const _: () = {
                    #miniserde_path ::make_place!(__Place);

                    impl #miniserde_path ::de::Deserialize for #name_ident {
                        fn begin(out: &mut ::std::option::Option<Self>) -> &mut dyn #miniserde_path ::de::Visitor {
                            __Place::new(out)
                        }
                    }

                    struct #builder_ident<'a> {
                        #(#builder_fields,)*
                        out: &'a mut ::std::option::Option<#name_ident>,
                    }

                    impl<'a> #miniserde_path ::de::Map for #builder_ident<'a> {
                        fn key(&mut self, k: &::std::primitive::str) -> #miniserde_path ::Result<&mut dyn #miniserde_path ::de::Visitor> {
                            match k {
                                #(#field_serialized_as => ::std::result::Result::Ok(#miniserde_path ::de::Deserialize::begin(&mut self.#field_idents)),)*
                                _ => ::std::result::Result::Ok(<dyn #miniserde_path ::de::Visitor>::ignore()),
                            }
                        }

                        fn finish(&mut self) -> #miniserde_path ::Result<()> {
                            #(let #field_idents = self.#field_idents.take().ok_or(#miniserde_path ::Error)?;)*
                            *self.out = ::std::option::Option::Some(#name_ident { #(#field_idents,)* });
                            ::std::result::Result::Ok(())
                        }
                    }

                    impl #miniserde_path ::de::Visitor for __Place<#name_ident> {
                        fn map(&mut self) -> #miniserde_path ::Result<::std::boxed::Box<dyn #miniserde_path ::de::Map + '_>> {
                            ::std::result::Result::Ok(::std::boxed::Box::new(#builder_ident {
                                #(#field_idents: ::std::option::Option::None,)*
                                out: &mut self.out,
                            }))
                        }
                    }
                };
            }
        })
    }

    fn field_builders(&self) -> Vec<ExecutableFieldBuilder<'a, S>> {
        self.executable_struct
            .fields
            .iter()
            .map(|field| {
                ExecutableFieldBuilder::new(
                    field,
                    self.config,
                    self.depth,
                    self.executable_struct.parent_name,
                    None,
                )
            })
            .collect()
    }
}
