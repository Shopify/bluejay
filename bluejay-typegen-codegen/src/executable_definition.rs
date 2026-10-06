use crate::{
    input::{parse_key_value_with, CustomScalarOverride},
    map_parser_errors, validation, CodeGenerator, Config, DocumentInput,
};
use bluejay_core::definition::SchemaDefinition;
use bluejay_parser::ast::{executable::ExecutableDocument, Parse as _};
use bluejay_validator::executable::{
    document::{BuiltinRulesValidator, Orchestrator},
    Cache,
};
use syn::{parse::Parse, parse2};

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
                parse_key_value_with(
                    input,
                    &mut custom_scalar_overrides,
                    CustomScalarOverride::parse_all,
                )?;
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
        .result
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

    let custom_scalar_overrides = CustomScalarOverride::validate_all(
        custom_scalar_overrides,
        config.borrow(),
        "Custom scalar overrides must correspond to a path in the query that is a custom scalar type",
        |path| paths_with_custom_scalar_type.contains(path),
    )?;

    let executable_types = ExecutableType::for_executable_document(
        &executable_document,
        config,
        custom_scalar_overrides,
    );

    Ok(executable_types
        .iter()
        .flat_map(|et| ExecutableTypeBuilder::build(et, config.code_generator()))
        .collect())
}
