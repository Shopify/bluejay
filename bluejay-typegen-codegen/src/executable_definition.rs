use crate::{map_parser_errors, validation, CodeGenerator, Config, DocumentInput};
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

struct Input {
    query: DocumentInput,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let query = input.parse()?;
        Ok(Self { query })
    }
}

pub(crate) fn generate_executable_definition<S: SchemaDefinition, C: CodeGenerator>(
    config: &Config<S, C>,
    configuration: proc_macro2::TokenStream,
) -> syn::Result<Vec<syn::Item>> {
    let Input { query } = parse2(configuration)?;

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
    let validation_errors: Vec<_> = Orchestrator::<_, _, validation::Rule<_, _>>::validate(
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

    let executable_types = ExecutableType::for_executable_document(&executable_document, config);

    Ok(executable_types
        .iter()
        .flat_map(|et| ExecutableTypeBuilder::build(et, config.code_generator()))
        .collect())
}
