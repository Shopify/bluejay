use bluejay_parser::{
    ast::{
        definition::{DefinitionDocument, SchemaDefinition},
        executable::ExecutableDocument,
        Parse,
    },
    Error,
};
use bluejay_validator::executable::{document::BuiltinRulesValidator, Cache};

#[test]
fn test_error() {
    with_schema(|schema_definition| {
        insta::glob!("test_data/executable/error/*.graphql", |path| {
            let input = std::fs::read_to_string(path).unwrap();
            let executable_document = ExecutableDocument::parse(input.as_str())
                .result
                .expect("Document had parse errors");
            let cache = Cache::new(&executable_document, &schema_definition);
            let errors =
                BuiltinRulesValidator::validate(&executable_document, &schema_definition, &cache);
            let formatted_errors = Error::format_errors(
                input.as_str(),
                path.file_name().and_then(|f| f.to_str()),
                errors,
            );
            insta::assert_snapshot!(formatted_errors);
        });
    });
}

#[test]
fn test_valid() {
    with_schema(|schema_definition| {
        insta::glob!("test_data/executable/valid/*.graphql", |path| {
            let input = std::fs::read_to_string(path).unwrap();
            let executable_document = ExecutableDocument::parse(input.as_str())
                .result
                .unwrap_or_else(|_| panic!("Document `{}` had parse errors", path.display()));
            let cache = Cache::new(&executable_document, &schema_definition);
            let errors: Vec<_> =
                BuiltinRulesValidator::validate(&executable_document, &schema_definition, &cache)
                    .collect();
            assert!(
                errors.is_empty(),
                "Document `{}` had validation errors:\n{}",
                path.display(),
                Error::format_errors(
                    input.as_str(),
                    path.file_name().and_then(|f| f.to_str()),
                    errors
                ),
            )
        });
    });
}

fn with_schema(f: fn(SchemaDefinition) -> ()) {
    let s = std::fs::read_to_string("tests/test_data/executable/schema.graphql").unwrap();
    let definition_document = DefinitionDocument::parse(s.as_str())
        .result
        .expect("Schema had parse errors");
    let schema_definition =
        SchemaDefinition::try_from(&definition_document).expect("Schema had errors");
    f(schema_definition)
}
