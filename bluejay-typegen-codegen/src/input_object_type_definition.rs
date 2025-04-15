use crate::attributes::doc_string;
use crate::builtin_scalar::{builtin_scalar_type, scalar_is_reference};
use crate::names::{enum_variant_ident, field_ident, type_ident};
use crate::{types, CodeGenerator, Config};
use bluejay_core::definition::{
    prelude::*, BaseInputTypeReference, EnumTypeDefinition, InputTypeReference,
    ScalarTypeDefinition, SchemaDefinition,
};
use bluejay_core::{AsIter, Directive};
use proc_macro2::Span;
use std::collections::HashSet;
use syn::parse_quote;

pub(crate) struct InputObjectTypeDefinitionBuilder<'a, S: SchemaDefinition, C: CodeGenerator> {
    config: &'a Config<'a, S, C>,
    input_object_type_definition: &'a S::InputObjectTypeDefinition,
}

impl<'a, S: SchemaDefinition, C: CodeGenerator> InputObjectTypeDefinitionBuilder<'a, S, C> {
    pub(crate) fn build(
        input_object_type_definition: &'a S::InputObjectTypeDefinition,
        config: &'a Config<'a, S, C>,
    ) -> Vec<syn::Item> {
        let instance = Self {
            config,
            input_object_type_definition,
        };

        if input_object_type_definition
            .directives()
            .map(|directives| {
                directives
                    .iter()
                    .any(|directive| directive.name() == "oneOf")
            })
            .unwrap_or(false)
        {
            instance.build_enum()
        } else {
            instance.build_struct()
        }
    }

    fn build_enum(&self) -> Vec<syn::Item> {
        let attributes = self.attributes();
        let name_ident = self.name_ident();
        let variant_idents = self.variant_idents();
        let field_description_attributes = self.field_description_attributes();
        let variant_types = self.variant_types();
        let field_serde_rename_attributes = self.field_serde_rename_attributes();
        let field_serde_borrow_attributes = self.field_serde_borrow_attributes();
        let lifetime = self.lifetime(self.input_object_type_definition);

        vec![parse_quote! {
            #(#attributes)*
            pub enum #name_ident #lifetime {
                #(
                    #field_description_attributes
                    #field_serde_rename_attributes
                    #field_serde_borrow_attributes
                    #variant_idents(#variant_types),
                )*
            }
        }]
    }

    fn build_struct(&self) -> Vec<syn::Item> {
        let attributes = self.attributes();
        let name_ident = self.name_ident();
        let field_idents = self.field_idents();
        let field_description_attributes = self.field_description_attributes();
        let field_types = self.field_types();
        let field_serde_rename_attributes = self.field_serde_rename_attributes();
        let field_serde_borrow_attributes = self.field_serde_borrow_attributes();
        let lifetime = self.lifetime(self.input_object_type_definition);

        vec![parse_quote! {
            #(#attributes)*
            pub struct #name_ident #lifetime {
                #(
                    #field_description_attributes
                    #field_serde_rename_attributes
                    #field_serde_borrow_attributes
                    pub #field_idents: #field_types,
                )*
            }
        }]
    }

    fn name_ident(&self) -> syn::Ident {
        type_ident(self.input_object_type_definition.name())
    }

    fn attributes(&self) -> Vec<syn::Attribute> {
        let mut attributes = Vec::new();
        attributes.extend(
            self.input_object_type_definition
                .description()
                .map(doc_string),
        );
        attributes.push(parse_quote! { #[derive(::std::clone::Clone, ::std::cmp::PartialEq, ::std::fmt::Debug, ::bluejay_typegen::serde::Serialize)] });
        attributes.push(parse_quote! { #[serde(crate = "bluejay_typegen::serde")] });

        attributes
    }

    fn variant_idents(&self) -> Vec<syn::Ident> {
        self.input_object_type_definition
            .input_field_definitions()
            .iter()
            .map(|ivd| enum_variant_ident(ivd.name()))
            .collect()
    }

    fn field_idents(&self) -> Vec<syn::Ident> {
        self.input_object_type_definition
            .input_field_definitions()
            .iter()
            .map(|ivd| field_ident(ivd.name()))
            .collect()
    }

    fn field_description_attributes(&self) -> Vec<Option<syn::Attribute>> {
        self.input_object_type_definition
            .input_field_definitions()
            .iter()
            .map(|ivd| ivd.description().map(doc_string))
            .collect()
    }

    fn field_types(&self) -> Vec<syn::Type> {
        self.input_object_type_definition
            .input_field_definitions()
            .iter()
            .map(|ivd| {
                self.type_for_input_value_definition(self.input_object_type_definition.name(), ivd)
            })
            .collect()
    }

    fn variant_types(&self) -> Vec<syn::Type> {
        self.input_object_type_definition
            .input_field_definitions()
            .iter()
            .map(|ivd| {
                // since we're building a oneOf enum, all types are optional, but we need to make
                // them required for the enum variant
                let required_type = match ivd.r#type().as_ref(self.config.schema_definition()) {
                    InputTypeReference::Base(base, _) => InputTypeReference::Base(base, true),
                    InputTypeReference::List(inner, _) => InputTypeReference::List(inner, true),
                };
                self.type_for_input_type(
                    required_type,
                    Some(self.input_object_type_definition.name()),
                    None,
                )
            })
            .collect()
    }

    fn field_serialized_as(&self) -> Vec<syn::LitStr> {
        self.input_object_type_definition
            .input_field_definitions()
            .iter()
            .map(|ivd| syn::LitStr::new(ivd.name(), Span::call_site()))
            .collect()
    }

    fn field_serde_rename_attributes(&self) -> Vec<syn::Attribute> {
        self.field_serialized_as()
            .iter()
            .map(|serialized_as| parse_quote!(#[serde(rename = #serialized_as)]))
            .collect()
    }

    fn field_serde_borrow_attributes(&self) -> Vec<Option<syn::Attribute>> {
        self.input_object_type_definition
            .input_field_definitions()
            .iter()
            .map(|ivd| {
                self.contains_reference_types(ivd.r#type(), &mut HashSet::new())
                    .then(|| parse_quote!(#[serde(borrow)]))
            })
            .collect()
    }

    fn lifetime(
        &self,
        input_object_type_definition: &'a S::InputObjectTypeDefinition,
    ) -> Option<syn::Generics> {
        (self.input_object_contains_reference_types(
            input_object_type_definition,
            &mut HashSet::new(),
        ))
        .then(|| parse_quote! { <'a> })
    }

    fn contains_non_list_reference(
        &self,
        target: &str,
        ty: InputTypeReference<'a, S::InputType>,
        visited: &mut HashSet<&'a str>,
    ) -> bool {
        match ty {
            InputTypeReference::Base(base, _) if base.name() == target => true,
            ty => match ty.base(self.config.schema_definition()) {
                BaseInputTypeReference::InputObject(iotd) => {
                    if visited.insert(iotd.name()) {
                        iotd.input_field_definitions().iter().any(|ivd| {
                            self.contains_non_list_reference(
                                target,
                                ivd.r#type().as_ref(self.config.schema_definition()),
                                visited,
                            )
                        })
                    } else {
                        false
                    }
                }
                _ => false,
            },
        }
    }

    fn contains_reference_types(
        &self,
        ty: &'a S::InputType,
        visited: &mut HashSet<&'a str>,
    ) -> bool {
        let base = ty.base(self.config.schema_definition());
        if !self.config.borrow() || !visited.insert(base.name()) {
            return false;
        }

        match base {
            BaseInputTypeReference::BuiltinScalar(bstd) => scalar_is_reference(bstd),
            BaseInputTypeReference::CustomScalar(cstd) => self.config.custom_scalar_borrows(cstd),
            BaseInputTypeReference::Enum(etd) => {
                self.config.enum_as_str(etd) && self.config.borrow()
            }
            BaseInputTypeReference::InputObject(iotd) => {
                self.input_object_contains_reference_types(iotd, visited)
            }
        }
    }

    fn input_object_contains_reference_types(
        &self,
        iotd: &'a S::InputObjectTypeDefinition,
        visited: &mut HashSet<&'a str>,
    ) -> bool {
        iotd.input_field_definitions()
            .iter()
            .any(|ivd| self.contains_reference_types(ivd.r#type(), visited))
    }

    fn type_for_base_input_type(&self, base: BaseInputTypeReference<S::InputType>) -> syn::Type {
        match base {
            BaseInputTypeReference::BuiltinScalar(bstd) => {
                builtin_scalar_type(bstd, self.config.borrow())
            }
            BaseInputTypeReference::InputObject(iotd) => {
                let ident = type_ident(iotd.name());
                let lifetime = self.lifetime(iotd);
                parse_quote! { #ident #lifetime }
            }
            BaseInputTypeReference::Enum(etd) => {
                if self.config.enum_as_str(etd) {
                    types::string(self.config.borrow())
                } else {
                    let ident = type_ident(etd.name());
                    parse_quote! { #ident }
                }
            }
            BaseInputTypeReference::CustomScalar(cstd) => {
                let ident = type_ident(cstd.name());
                let lifetime: Option<syn::Generics> = self
                    .config
                    .custom_scalar_borrows(cstd)
                    .then(|| parse_quote! { <'a> });
                parse_quote! { #ident #lifetime }
            }
        }
    }

    fn type_for_input_type(
        &self,
        ty: InputTypeReference<S::InputType>,
        parent_type_name: Option<&str>,
        has_default_value: Option<bool>,
    ) -> syn::Type {
        let required = has_default_value.map_or_else(
            || ty.is_required(),
            |has_default_value| !has_default_value && ty.is_required(),
        );
        match ty {
            InputTypeReference::Base(base, _) => {
                let mut inner = self.type_for_base_input_type(base);
                if let Some(parent_type_name) = parent_type_name {
                    if self.contains_non_list_reference(parent_type_name, ty, &mut HashSet::new()) {
                        inner = parse_quote! { ::std::boxed::Box<#inner> };
                    }
                }
                if required {
                    inner
                } else {
                    crate::types::option(inner)
                }
            }
            InputTypeReference::List(inner, _) => {
                let inner_ty = crate::types::vec(self.type_for_input_type(
                    inner.as_ref(self.config.schema_definition()),
                    None,
                    None,
                ));
                if required {
                    inner_ty
                } else {
                    crate::types::option(inner_ty)
                }
            }
        }
    }

    fn type_for_input_value_definition(
        &self,
        parent_type_name: &str,
        ivd: &S::InputValueDefinition,
    ) -> syn::Type {
        self.type_for_input_type(
            ivd.r#type().as_ref(self.config.schema_definition()),
            Some(parent_type_name),
            Some(ivd.default_value().is_some()),
        )
    }
}
