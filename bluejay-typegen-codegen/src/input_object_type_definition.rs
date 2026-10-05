use crate::attributes::doc_string;
use crate::input_type::{base_input_type, input_field_borrows, input_object_lifetime};
use crate::names::{enum_variant_ident, field_ident, type_ident};
use crate::{CodeGenerator, Config};
use bluejay_core::definition::{
    prelude::*, BaseInputTypeReference, InputTypeReference, SchemaDefinition,
};
use bluejay_core::{AsIter, Directive};
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
        let attributes = self.attributes_for_enum();
        let name_ident = self.name_ident();
        let lifetime = input_object_lifetime(self.config, self.input_object_type_definition);

        let variants: Vec<syn::Variant> = self
            .input_object_type_definition
            .input_field_definitions()
            .iter()
            .map(|ivd| {
                let variant_ident = enum_variant_ident(ivd.name());
                let variant_type = self.variant_type(ivd);
                let description_attribute = ivd.description().map(doc_string);
                let variant_attributes = self
                    .config
                    .code_generator()
                    .attributes_for_one_of_input_object_field(ivd, false);

                parse_quote! {
                    #description_attribute
                    #(#variant_attributes)*
                    #variant_ident(#variant_type)
                }
            })
            .collect();

        let mut items = vec![parse_quote! {
            #(#attributes)*
            pub enum #name_ident #lifetime {
                #(
                    #variants,
                )*
            }
        }];

        items.extend(
            self.config
                .code_generator()
                .additional_impls_for_one_of_input_object(self.input_object_type_definition)
                .into_iter()
                .map(syn::Item::Impl),
        );

        items
    }

    fn build_struct(&self) -> Vec<syn::Item> {
        let attributes = self.attributes_for_struct();
        let name_ident = self.name_ident();
        let lifetime = input_object_lifetime(self.config, self.input_object_type_definition);

        let fields: Vec<syn::Field> = self
            .input_object_type_definition
            .input_field_definitions()
            .iter()
            .map(|ivd| {
                let field_ident = field_ident(ivd.name());
                let field_type = self.type_for_input_value_definition(ivd);
                let description_attribute = ivd.description().map(doc_string);
                let field_attributes = self
                    .config
                    .code_generator()
                    .attributes_for_input_object_field(
                        ivd,
                        input_field_borrows(
                            self.config,
                            self.input_object_type_definition,
                            ivd,
                            &mut HashSet::new(),
                        ),
                    );

                parse_quote! {
                    #description_attribute
                    #(#field_attributes)*
                    pub #field_ident: #field_type
                }
            })
            .collect();

        let mut items = vec![parse_quote! {
            #(#attributes)*
            pub struct #name_ident #lifetime {
                #(#fields,)*
            }
        }];

        items.extend(
            self.config
                .code_generator()
                .additional_impls_for_input_object(self.input_object_type_definition)
                .into_iter()
                .map(syn::Item::Impl),
        );

        items
    }

    fn name_ident(&self) -> syn::Ident {
        type_ident(self.input_object_type_definition.name())
    }

    fn attributes_for_struct(&self) -> Vec<syn::Attribute> {
        self.input_object_type_definition
            .description()
            .map(doc_string)
            .into_iter()
            .chain(
                self.config
                    .code_generator()
                    .attributes_for_input_object(self.input_object_type_definition),
            )
            .collect()
    }

    fn attributes_for_enum(&self) -> Vec<syn::Attribute> {
        self.input_object_type_definition
            .description()
            .map(doc_string)
            .into_iter()
            .chain(
                self.config
                    .code_generator()
                    .attributes_for_one_of_input_object(self.input_object_type_definition),
            )
            .collect()
    }

    fn contains_non_list_reference(
        &self,
        target: &str,
        ty: InputTypeReference<'a, S>,
        visited: &mut HashSet<&'a str>,
    ) -> bool {
        match ty {
            InputTypeReference::Base(base, _) if base.name() == target => true,
            ty => match ty.base(self.config.schema_definition()) {
                BaseInputTypeReference::InputObject(iotd) if visited.insert(iotd.name()) => {
                    iotd.input_field_definitions().iter().any(|ivd| {
                        self.contains_non_list_reference(
                            target,
                            ivd.r#type().as_ref(self.config.schema_definition()),
                            visited,
                        )
                    })
                }
                _ => false,
            },
        }
    }

    /// The type for `ty`, with `custom_scalar_override` in place of its base type if given.
    fn type_for_input_type(
        &self,
        ty: InputTypeReference<S>,
        parent_type_name: Option<&str>,
        has_default_value: Option<bool>,
        custom_scalar_override: Option<&syn::Type>,
    ) -> syn::Type {
        let required = has_default_value.map_or_else(
            || ty.is_required(),
            |has_default_value| !has_default_value && ty.is_required(),
        );
        match ty {
            InputTypeReference::Base(base, _) => {
                let mut inner = custom_scalar_override
                    .cloned()
                    .unwrap_or_else(|| base_input_type(self.config, base, 0));
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
                    custom_scalar_override,
                ));
                if required {
                    inner_ty
                } else {
                    crate::types::option(inner_ty)
                }
            }
        }
    }

    fn type_for_input_value_definition(&self, ivd: &S::InputValueDefinition) -> syn::Type {
        self.type_for_input_type(
            ivd.r#type().as_ref(self.config.schema_definition()),
            Some(self.input_object_type_definition.name()),
            Some(ivd.default_value().is_some()),
            self.custom_scalar_override(ivd),
        )
    }

    fn custom_scalar_override(&self, ivd: &S::InputValueDefinition) -> Option<&syn::Type> {
        self.config
            .custom_scalar_override(self.input_object_type_definition, ivd)
            .map(|custom_scalar_override| custom_scalar_override.r#type())
    }

    fn variant_type(&self, ivd: &S::InputValueDefinition) -> syn::Type {
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
            self.custom_scalar_override(ivd),
        )
    }
}
