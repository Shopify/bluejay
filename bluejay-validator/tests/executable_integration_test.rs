use bluejay_parser::{
    ast::definition::{DefinitionDocument, SchemaDefinition},
    ast::executable::ExecutableDocument,
    Error,
};
use bluejay_validator::executable::RulesValidator;

#[test]
fn test_error() {
    with_schema(|schema_definition| {
        insta::glob!("test_data/executable/error/*.graphql", |path| {
            let input = std::fs::read_to_string(path).unwrap();
            let executable_document =
                ExecutableDocument::parse(input.as_str()).expect("Document had parse errors");
            let errors = RulesValidator::validate(&executable_document, &schema_definition);
            let formatted_errors = Error::format_errors(input.as_str(), errors);
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
                .expect(format!("Document `{}` had parse errors", path.display()).as_str());
            let errors: Vec<_> = RulesValidator::validate(&executable_document, &schema_definition)
                .into_iter()
                .collect();
            assert!(
                errors.is_empty(),
                "Document `{}` had validation errors",
                path.display()
            )
        });
    });
}

fn with_schema<F: Fn(SchemaDefinition) -> ()>(f: F) {
    let s = std::fs::read_to_string("tests/test_data/executable/schema.graphql").unwrap();
    let definition_document =
        DefinitionDocument::parse(s.as_str()).expect("Schema had parse errors");
    let schema_definition =
        SchemaDefinition::try_from(&definition_document).expect("Schema had errors");
    f(schema_definition)
}
