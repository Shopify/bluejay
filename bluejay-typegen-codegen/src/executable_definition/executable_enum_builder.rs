use crate::executable_definition::{
    ExecutableEnum, ExecutableEnumVariantBuilder, ExecutableTypeBuilder,
};
use crate::{
    attributes::doc_string,
    input::Codec,
    names::{module_ident, type_ident},
    types::option,
    Config,
};
use bluejay_core::{definition::SchemaDefinition, AsIter};
use quote::format_ident;
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
        items.extend(instance.miniserde_deserialize_impl());

        items
    }

    fn name_ident(&self) -> syn::Ident {
        type_ident(self.executable_enum.parent_name)
    }

    fn attributes(&self) -> Vec<syn::Attribute> {
        let mut attributes = Vec::new();
        attributes.extend(self.executable_enum.description.map(doc_string));
        attributes.push(parse_quote! { #[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug)] });
        let serde_path = self.config.serde_path();
        let serde_path_lit_str = self.config.serde_path_lit_str();

        match self.config.codec() {
            Codec::Serde => {
                attributes.push(parse_quote! { #[derive(#serde_path ::Deserialize)] });
                attributes.push(parse_quote! { #[serde(crate = #serde_path_lit_str)] });
                attributes.push(parse_quote! { #[serde(tag = "__typename")] });
            }
            Codec::Miniserde => {}
        }

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
                ExecutableEnumVariantBuilder::build_other_variant(self.config),
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

    fn miniserde_deserialize_impl(&self) -> Option<syn::Item> {
        (self.config.codec() == Codec::Miniserde).then(|| {
            let name_ident = self.name_ident();
            let builder_ident = format_ident!("{}Builder", name_ident);
            let builder_state_ident = format_ident!("{}BuilderState", name_ident);

            let variant_builders = self.variant_builders();

            let builder_state_variants = variant_builders
                .iter()
                .map(|variant_builder| {
                    let name_ident = variant_builder.name_ident();

                    let field_builders = variant_builder.field_builders();

                    let variant_fields = field_builders.iter().map(|field_builder| {
                        let name_ident = field_builder.name_ident();
                        let type_path = option(field_builder.type_path());
                        parse_quote! { #name_ident: #type_path }
                    }).collect::<Vec<syn::Field>>();

                    parse_quote! {
                        #name_ident { #(#variant_fields,)* }
                    }
                })
                .chain(
                    std::iter::once(parse_quote! { Other }),
                )
                .collect::<Vec<syn::Variant>>();

            let typename_to_builder_variant = variant_builders
                .iter()
                .map(|variant_builder| {
                    let serialized_as = variant_builder.serialized_as();
                    let name_ident = variant_builder.name_ident();

                    let field_idents = variant_builder.field_builders().iter().map(|field_builder| {
                        field_builder.name_ident()
                    }).collect::<Vec<syn::Ident>>();

                    parse_quote! {
                        #serialized_as => {
                            self.out = ::std::option::Option::Some(#builder_state_ident::#name_ident { #(#field_idents: ::std::option::Option::None,)* });
                            ::std::result::Result::Ok(())
                        }
                    }
                }).chain(std::iter::once(
                    parse_quote! {
                        _ => {
                            self.out = ::std::option::Option::Some(#builder_state_ident::Other);
                            ::std::result::Result::Ok(())
                        }
                    }
                ))
                .collect::<Vec<syn::Arm>>();

            let miniserde_path = self.config.miniserde_path();

            let state_key_arm = variant_builders
                .iter()
                .map(|variant_builder| {
                    let variant_ident = variant_builder.name_ident();
                    let field_builders = variant_builder.field_builders();
                    let field_idents = field_builders
                        .iter()
                        .map(|field_builder| field_builder.name_ident())
                        .collect::<Vec<_>>();
                    let field_serialized_as = field_builders
                        .iter()
                        .map(|field_builder| field_builder.serialized_as())
                        .collect::<Vec<_>>();

                    parse_quote! {
                        #builder_state_ident::#variant_ident { #(#field_idents,)* } => {
                            match k {
                                #(#field_serialized_as => ::std::result::Result::Ok(#miniserde_path ::de::Deserialize::begin(#field_idents)),)*
                                _ => ::std::result::Result::Err(#miniserde_path ::Error)
                            }
                        }
                    }
                })
                .chain(std::iter::once(
                    parse_quote! {
                        #builder_state_ident::Other => ::std::result::Result::Err(#miniserde_path ::Error)
                    }
                ))
                .collect::<Vec<syn::Arm>>();

            let state_finish_arm = variant_builders
                .iter()
                .map(|variant_builder| {
                    let variant_ident = variant_builder.name_ident();
                    let field_idents = variant_builder.field_builders()
                        .iter()
                        .map(|field_builder| field_builder.name_ident())
                        .collect::<Vec<_>>();

                    parse_quote! {
                        #builder_state_ident::#variant_ident { #(mut #field_idents,)* } => {
                            #name_ident::#variant_ident { #(#field_idents: #field_idents.take().ok_or(#miniserde_path ::Error)?,)* }
                        }
                    }
                })
                .chain(std::iter::once(
                    parse_quote! {
                        #builder_state_ident::Other => #name_ident::Other
                    }
                ))
                .collect::<Vec<syn::Arm>>();

            parse_quote! {
                const _: () = {
                    #miniserde_path ::make_place!(__Place);

                    impl #miniserde_path ::de::Deserialize for #name_ident {
                        fn begin(out: &mut ::std::option::Option<Self>) -> &mut dyn #miniserde_path ::de::Visitor {
                            __Place::new(out)
                        }
                    }

                    impl #miniserde_path ::de::Visitor for __Place<#name_ident> {
                        fn map(&mut self) -> #miniserde_path ::Result<::std::boxed::Box<dyn #miniserde_path ::de::Map + '_>> {
                            ::std::result::Result::Ok(::std::boxed::Box::new(#builder_ident {
                                state: ::std::option::Option::None,
                                out: &mut self.out,
                            }))
                        }
                    }

                    enum #builder_state_ident {
                        #(#builder_state_variants,)*
                    }

                    impl #miniserde_path ::de::Visitor for __Place<#builder_state_ident> {
                        fn string(&mut self, s: &::std::primitive::str) -> #miniserde_path ::Result<()> {
                            match s {
                                #(#typename_to_builder_variant,)*
                            }
                        }
                    }

                    struct #builder_ident<'a> {
                        state: ::std::option::Option<#builder_state_ident>,
                        out: &'a mut ::std::option::Option<#name_ident>,
                    }

                    impl #miniserde_path ::de::Deserialize for #builder_state_ident {
                        fn begin(out: &mut ::std::option::Option<Self>) -> &mut dyn #miniserde_path ::de::Visitor {
                            __Place::new(out)
                        }
                    }

                    impl<'a> #miniserde_path ::de::Map for #builder_ident<'a> {
                        fn key(&mut self, k: &::std::primitive::str) -> #miniserde_path ::Result<&mut dyn #miniserde_path ::de::Visitor> {
                            if let ::std::option::Option::Some(ref mut state) = self.state {
                                match state {
                                    #(#state_key_arm,)*
                                }
                            } else if k == "__typename" {
                                ::std::result::Result::Ok(#miniserde_path ::de::Deserialize::begin(&mut self.state))
                            } else {
                                ::std::result::Result::Err(#miniserde_path ::Error)
                            }
                        }

                        fn finish(&mut self) -> #miniserde_path ::Result<()> {
                            let mut state = self.state.take().ok_or(#miniserde_path ::Error)?;

                            *self.out = ::std::option::Option::Some(match state {
                                #(#state_finish_arm,)*
                            });
                            ::std::result::Result::Ok(())
                        }
                    }
                };
            }
        })
    }

    fn variant_builders(&self) -> Vec<ExecutableEnumVariantBuilder<'a, S>> {
        self.executable_enum
            .variants
            .iter()
            .map(|variant| {
                ExecutableEnumVariantBuilder::new(
                    self.config,
                    variant,
                    self.depth,
                    self.executable_enum.parent_name,
                )
            })
            .collect()
    }
}
