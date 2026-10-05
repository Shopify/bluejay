use crate::attributes::doc_string;
use crate::input_type::{base_input_type, base_input_type_borrows};
use crate::names::{field_ident, variables_type_ident};
use crate::{types, CodeGenerator, Config};
use bluejay_core::definition::{BaseInputTypeReference, SchemaDefinition};
use bluejay_core::executable::{ExecutableDocument, OperationDefinition, VariableDefinition};
use bluejay_core::AsIter;
use bluejay_validator::executable::{document::VariableDefinitionInputType, Cache};
use std::collections::HashSet;
use syn::parse_quote;

/// Builds the struct holding the variables of an operation, which sits next to the struct for the operation.
pub(crate) struct VariablesStructBuilder<
    'a,
    E: ExecutableDocument,
    S: SchemaDefinition,
    C: CodeGenerator,
> {
    config: &'a Config<'a, S, C>,
    cache: &'a Cache<'a, E, S>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition, C: CodeGenerator>
    VariablesStructBuilder<'a, E, S, C>
{
    /// Builds nothing if the operation does not define any variables.
    pub(crate) fn build(
        operation_definition: &'a E::OperationDefinition,
        config: &'a Config<'a, S, C>,
        cache: &'a Cache<'a, E, S>,
    ) -> Vec<syn::Item> {
        let operation_definition_reference = operation_definition.as_ref();
        let Some(variable_definitions) = operation_definition_reference
            .variable_definitions()
            .filter(|variable_definitions| !variable_definitions.is_empty())
        else {
            return Vec::new();
        };

        let instance = Self { config, cache };

        let (fields, borrows): (Vec<syn::Field>, Vec<bool>) = variable_definitions
            .iter()
            .map(|variable_definition| instance.field(variable_definition))
            .unzip();

        let attributes = config
            .code_generator()
            .attributes_for_variables_struct(operation_definition);
        let name_ident = variables_type_ident(operation_definition_reference.name());
        let lifetime: Option<syn::Generics> =
            borrows.contains(&true).then(|| parse_quote! { <'a> });

        let mut items = vec![parse_quote! {
            #(#attributes)*
            pub struct #name_ident #lifetime {
                #(#fields,)*
            }
        }];

        items.extend(
            config
                .code_generator()
                .additional_impls_for_variables_struct(operation_definition)
                .into_iter()
                .map(syn::Item::Impl),
        );

        items
    }

    /// The field for the variable, and whether its type borrows.
    fn field(&self, variable_definition: &'a E::VariableDefinition) -> (syn::Field, bool) {
        let input_type = self
            .cache
            .variable_definition_input_type(variable_definition.r#type())
            .expect("Variable types are validated to be input types");
        let borrows =
            base_input_type_borrows(self.config, Self::base(input_type), &mut HashSet::new());

        let field_ident = field_ident(variable_definition.variable());
        let field_type = self.type_for(input_type, variable_definition.default_value().is_some());
        let description_attribute = variable_definition.description().map(doc_string);
        let field_attributes = self
            .config
            .code_generator()
            .attributes_for_variables_struct_field(variable_definition, borrows);

        let field = parse_quote! {
            #description_attribute
            #(#field_attributes)*
            pub #field_ident: #field_type
        };

        (field, borrows)
    }

    /// A variable with a default value can be omitted even if its type is required, so its type is optional.
    fn type_for(
        &self,
        input_type: &VariableDefinitionInputType<'a, S>,
        has_default_value: bool,
    ) -> syn::Type {
        let (inner, required) = match input_type {
            // the struct is at the root of the query module, one level below the schema definition module
            VariableDefinitionInputType::Base(base, required) => {
                (base_input_type(self.config, *base, 1), *required)
            }
            VariableDefinitionInputType::List(inner, required) => {
                (types::vec(self.type_for(inner, false)), *required)
            }
        };

        if required && !has_default_value {
            inner
        } else {
            types::option(inner)
        }
    }

    fn base(input_type: &VariableDefinitionInputType<'a, S>) -> BaseInputTypeReference<'a, S> {
        match input_type {
            VariableDefinitionInputType::Base(base, _) => *base,
            VariableDefinitionInputType::List(inner, _) => Self::base(inner),
        }
    }
}
