use crate::{
    input::parse_key_value_with, map_parser_errors, validation, CodeGenerator, Config,
    DocumentInput,
};
use bluejay_core::definition::SchemaDefinition;
use bluejay_parser::ast::{executable::ExecutableDocument, Parse as _};
use bluejay_validator::executable::{
    document::{BuiltinRulesValidator, Orchestrator},
    Cache,
};
use itertools::{Either, Itertools};
use syn::{parse::Parse, parse2, spanned::Spanned};

mod executable_enum_builder;
mod executable_enum_variant_builder;
mod executable_struct_builder;
mod executable_type_builder;
mod intermediate_representation;

use executable_enum_builder::ExecutableEnumBuilder;
use executable_enum_variant_builder::ExecutableEnumVariantBuilder;
use executable_struct_builder::ExecutableStructBuilder;
use executable_type_builder::ExecutableTypeBuilder;
pub use intermediate_representation::{
    ExecutableEnum, ExecutableField, ExecutableStruct, ExecutableType, WrappedExecutableType,
};

mod kw {
    syn::custom_keyword!(custom_scalar_overrides);
}

pub(crate) struct CustomScalarOverride {
    graphql_path_token: syn::LitStr,
    graphql_path: Vec<String>,
    type_token: syn::Type,
    borrows: bool,
}

impl Parse for CustomScalarOverride {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let graphql_path_token = input.parse()?;
        let graphql_path = Self::graphql_path(&graphql_path_token);
        input.parse::<syn::Token![=>]>()?;
        let type_token = input.parse()?;

        let borrows = Self::type_borrows(&type_token)?;

        Ok(Self {
            graphql_path_token,
            graphql_path,
            type_token,
            borrows,
        })
    }
}

impl CustomScalarOverride {
    fn graphql_path(lit_str: &syn::LitStr) -> Vec<String> {
        lit_str.value().split('.').map(|s| s.to_string()).collect()
    }

    fn type_borrows(ty: &syn::Type) -> syn::Result<bool> {
        let syn::Type::Path(path) = ty else {
            // in the future, we could support arrays and tuples, but it is complex to
            // resolve the relative path with composite types like these so for simplicity
            // we don't support them
            return Err(syn::Error::new(
                ty.span(),
                "Unsupported type for custom scalar overrides",
            ));
        };

        let Some(last_segment) = path.path.segments.last() else {
            return Err(syn::Error::new(
                path.span(),
                "Path must have at least one segment",
            ));
        };

        let path_arguments = match &last_segment.arguments {
            syn::PathArguments::None => return Ok(false),
            syn::PathArguments::AngleBracketed(bracketed) => bracketed,
            syn::PathArguments::Parenthesized(parenthesized) => {
                return Err(syn::Error::new(
                    parenthesized.span(),
                    "Paths for custom scalar overrides must not contain parenthesized generic arguments",
                ));
            }
        };

        if path_arguments.args.len() != 1
            || !matches!(
                path_arguments.args.first(),
                Some(syn::GenericArgument::Lifetime(lifetime)) if lifetime.ident != "'a"
            )
        {
            return Err(syn::Error::new(
                ty.span(),
                "Paths for custom scalar overrides with generic arguments must contain a single lifetime parameter 'a",
            ));
        }

        Ok(true)
    }

    fn r#type(&self) -> &syn::Type {
        &self.type_token
    }
}

struct Input {
    query: DocumentInput,
    custom_scalar_overrides:
        Option<syn::punctuated::Punctuated<CustomScalarOverride, syn::Token![,]>>,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let query = input.parse()?;

        let mut custom_scalar_overrides = None;

        while !input.is_empty() {
            input.parse::<syn::Token![,]>()?;
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::custom_scalar_overrides) {
                parse_key_value_with(input, &mut custom_scalar_overrides, |input| {
                    let content;
                    syn::braced!(content in input);
                    syn::punctuated::Punctuated::parse_terminated(&content)
                })?;
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(Self {
            query,
            custom_scalar_overrides,
        })
    }
}

pub(crate) fn generate_executable_definition<S: SchemaDefinition, C: CodeGenerator>(
    config: &Config<S, C>,
    configuration: proc_macro2::TokenStream,
) -> syn::Result<Vec<syn::Item>> {
    let Input {
        query,
        custom_scalar_overrides,
    } = parse2(configuration)?;

    let (contents, path) = query.read_to_string_and_path()?;

    let executable_document = ExecutableDocument::parse(&contents)
        .map_err(|errors| map_parser_errors(&query, &contents, path.as_deref(), errors))?;
    let validation_cache = Cache::new(&executable_document, config.schema_definition());
    let validation_errors: Vec<_> = BuiltinRulesValidator::validate(
        &executable_document,
        config.schema_definition(),
        &validation_cache,
    )
    .collect();
    if !validation_errors.is_empty() {
        return Err(map_parser_errors(
            &query,
            &contents,
            path.as_deref(),
            validation_errors,
        ));
    }
    let (validation_errors, paths_with_custom_scalar_type) = Orchestrator::<
        _,
        _,
        (
            validation::Rule<_, _>,
            validation::PathsWithCustomScalarType<_>,
        ),
    >::validate_and_analyze(
        &executable_document,
        config.schema_definition(),
        &validation_cache,
    );

    let validation_errors: Vec<_> = validation_errors.collect();

    if !validation_errors.is_empty() {
        return Err(map_parser_errors(
            &query,
            &contents,
            path.as_deref(),
            validation_errors,
        ));
    }

    let custom_scalar_overrides: Vec<CustomScalarOverride> = custom_scalar_overrides
        .map(|c| c.into_iter().collect())
        .unwrap_or_default();

    let (valid_custom_scalar_overrides, custom_scalar_override_errors): (Vec<_>, Vec<syn::Error>) =
        custom_scalar_overrides
            .into_iter()
            .partition_map(|c| {
                if paths_with_custom_scalar_type.contains(&c.graphql_path) {
                    if c.borrows && !config.borrow() {
                        Either::Right(syn::Error::new(
                            c.type_token.span(),
                            "Custom scalar overrides must not borrow if the `borrow` option is not enabled",
                        ))
                    } else {
                        Either::Left(c)
                    }
                } else {
                    Either::Right(syn::Error::new(
                        c.graphql_path_token.span(),
                        "Custom scalar overrides must correspond to a path in the query that is a custom scalar type",
                    ))
                }
            });

    if let Some(combined_error) =
        custom_scalar_override_errors
            .into_iter()
            .reduce(|mut acc, error| {
                acc.combine(error);
                acc
            })
    {
        return Err(combined_error);
    }

    let executable_types = ExecutableType::for_executable_document(
        &executable_document,
        config,
        valid_custom_scalar_overrides,
    );

    Ok(executable_types
        .iter()
        .flat_map(|et| ExecutableTypeBuilder::build(et, config.code_generator()))
        .collect())
}
