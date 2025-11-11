use bluejay_parser::{
    ast::{
        definition::{DefinitionDocument, SchemaDefinition},
        Parse,
    },
    Error,
};
use bluejay_validator::definition::BuiltinRulesValidator;

#[test]
fn test_error() {
    insta::glob!("test_data/definition/error/*.graphql", |path| {
        let input = std::fs::read_to_string(path).unwrap();
        let definition_document: DefinitionDocument = DefinitionDocument::parse(&input)
            .result
            .unwrap_or_else(|_| panic!("Schema `{}` had parse errors", path.display()));
        let schema_definition = SchemaDefinition::try_from(&definition_document)
            .unwrap_or_else(|_| panic!("Schema `{}` had coercion errors", path.display()));

        let errors: Vec<_> = BuiltinRulesValidator::validate(&schema_definition).collect();

        let formatted_errors =
            Error::format_errors(&input, path.file_name().and_then(|f| f.to_str()), errors);
        insta::assert_snapshot!(formatted_errors);
    });
}

#[test]
fn test_valid() {
    insta::glob!("test_data/definition/valid/*.graphql", |path| {
        let input = std::fs::read_to_string(path).unwrap();
        let definition_document: DefinitionDocument = DefinitionDocument::parse(&input)
            .result
            .unwrap_or_else(|_| panic!("Schema `{}` had parse errors", path.display()));
        let schema_definition = SchemaDefinition::try_from(&definition_document)
            .unwrap_or_else(|_| panic!("Schema `{}` had coercion errors", path.display()));

        let errors: Vec<_> = BuiltinRulesValidator::validate(&schema_definition).collect();

        assert!(
            errors.is_empty(),
            "Schema `{}` had validation errors:\n{}",
            path.display(),
            Error::format_errors(&input, path.file_name().and_then(|f| f.to_str()), errors),
        )
    });
}
