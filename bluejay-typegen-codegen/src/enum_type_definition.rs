use crate::attributes::doc_string;
use crate::names::{enum_variant_ident, type_ident};
use crate::Codec;
use crate::Config;
use bluejay_core::definition::{EnumTypeDefinition, EnumValueDefinition, SchemaDefinition};
use bluejay_core::AsIter;
use proc_macro2::Span;
use syn::parse_quote;

pub(crate) struct EnumTypeDefinitionBuilder<'a, S: SchemaDefinition> {
    config: &'a Config<'a, S>,
    enum_type_definition: &'a S::EnumTypeDefinition,
}

impl<'a, S: SchemaDefinition> EnumTypeDefinitionBuilder<'a, S> {
    pub(crate) fn build(
        enum_type_definition: &'a S::EnumTypeDefinition,
        config: &'a Config<'a, S>,
    ) -> Vec<syn::Item> {
        let instance = Self {
            config,
            enum_type_definition,
        };

        let name_ident = instance.name_ident();
        let variant_description_attributes = instance.variant_description_attributes();
        let variant_serde_rename_attributes = instance.variant_serde_rename_attributes();
        let variant_idents = instance.variant_idents();
        let attributes = instance.attributes();
        let other_variant = instance.other_variant();

        let mut items = vec![parse_quote! {
            #(#attributes)*
            pub enum #name_ident {
                #(
                    #variant_description_attributes
                    #variant_serde_rename_attributes
                    #variant_idents,
                )*
                #other_variant,
            }
        }];

        items.extend(instance.miniserde_serialize_impl());
        items.extend(instance.miniserde_deserialize_impl());

        items
    }

    fn name_ident(&self) -> syn::Ident {
        type_ident(self.enum_type_definition.name())
    }

    fn variant_idents(&self) -> Vec<syn::Ident> {
        self.enum_type_definition
            .enum_value_definitions()
            .iter()
            .map(|evd| enum_variant_ident(evd.name()))
            .collect()
    }

    fn variant_description_attributes(&self) -> Vec<Option<syn::Attribute>> {
        self.enum_type_definition
            .enum_value_definitions()
            .iter()
            .map(|evd| evd.description().map(doc_string))
            .collect()
    }

    fn variant_serialized_as(&self) -> Vec<syn::LitStr> {
        self.enum_type_definition
            .enum_value_definitions()
            .iter()
            .map(|evd| syn::LitStr::new(evd.name(), Span::call_site()))
            .collect()
    }

    fn variant_serde_rename_attributes(&self) -> Vec<Option<syn::Attribute>> {
        self.variant_serialized_as()
            .iter()
            .map(|serialized_as| {
                (self.config.codec() == Codec::Serde)
                    .then(|| parse_quote!(#[serde(rename = #serialized_as)]))
            })
            .collect()
    }

    fn attributes(&self) -> Vec<syn::Attribute> {
        let mut attributes = Vec::new();
        attributes.extend(self.enum_type_definition.description().map(doc_string));
        attributes.push(parse_quote! { #[derive(::std::clone::Clone, ::std::marker::Copy, ::std::cmp::PartialEq, ::std::cmp::Eq, ::std::fmt::Debug)] });
        let serde_path = self.config.serde_path();
        let serde_path_lit_str = self.config.serde_path_lit_str();

        match self.config.codec() {
            Codec::Serde => attributes.extend([
                parse_quote! { #[derive(#serde_path ::Serialize, #serde_path ::Deserialize)] },
                parse_quote! { #[serde(crate = #serde_path_lit_str)] },
            ]),
            Codec::Miniserde => {}
        }

        attributes
    }

    fn miniserde_serialize_impl(&self) -> Option<syn::Item> {
        (self.config.codec() == Codec::Miniserde).then(|| {
            let name_ident = self.name_ident();
            let variant_idents = self.variant_idents();
            let variant_serialized_as = self.variant_serialized_as();
            let miniserde_path = self.config.miniserde_path();

            parse_quote! {
                const _: () = {
                    impl #miniserde_path ::ser::Serialize for #name_ident {
                        fn begin(&self) -> #miniserde_path ::ser::Fragment {
                            #miniserde_path ::ser::Fragment::Str(::std::borrow::Cow::Borrowed(match self {
                                #(
                                    #name_ident::#variant_idents => #variant_serialized_as,
                                )*
                                #name_ident::Other => ::std::panic!("Cannot serialize Other variant"),
                            }))
                        }
                    }
                };
            }
        })
    }

    fn miniserde_deserialize_impl(&self) -> Option<syn::Item> {
        (self.config.codec() == Codec::Miniserde).then(|| {
            let name_ident = self.name_ident();
            let variant_idents = self.variant_idents();
            let variant_serialized_as = self.variant_serialized_as();
            let miniserde_path = self.config.miniserde_path();

            parse_quote! {
                const _: () = {
                    #miniserde_path ::make_place!(__Place);

                    impl #miniserde_path ::de::Deserialize for #name_ident {
                        fn begin(out: &mut ::std::option::Option<Self>) -> &mut dyn #miniserde_path ::de::Visitor {
                            __Place::new(out)
                        }
                    }

                    impl #miniserde_path ::de::Visitor for __Place<#name_ident> {
                        fn string(&mut self, s: &::std::primitive::str) -> #miniserde_path ::Result<()> {
                            match s {
                                #(
                                    #variant_serialized_as => {
                                        self.out = ::std::option::Option::Some(#name_ident::#variant_idents);
                                        ::std::result::Result::Ok(())
                                    },
                                )*
                                _ => {
                                    self.out = ::std::option::Option::Some(#name_ident::Other);
                                    ::std::result::Result::Ok(())
                                },
                            }
                        }
                    }
                };
            }
        })
    }

    fn other_variant(&self) -> syn::Variant {
        let attributes: Vec<syn::Attribute> = match self.config.codec() {
            Codec::Serde => vec![parse_quote!(#[serde(other)])],
            Codec::Miniserde => Vec::new(),
        };

        parse_quote! {
            #(#attributes)*
            Other
        }
    }
}
