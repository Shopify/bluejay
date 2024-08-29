use bluejay_parser::{
    ast::{
        definition::{DefinitionDocument, SchemaDefinition},
        Parse,
    },
    Error,
};

#[test]
fn test_error() {
    insta::glob!("test_data/schema_definition/error/*.graphql", |path| {
        let input = std::fs::read_to_string(path).unwrap();
        let definition_document: Result<DefinitionDocument, _> =
            DefinitionDocument::parse(input.as_str());
        let errors = match definition_document {
            Ok(definition_document) => match SchemaDefinition::try_from(&definition_document) {
                Ok(_) => panic!("Document did not have any errors"),
                Err(errors) => errors.into_iter().map(Error::from).collect(),
            },
            Err(errors) => errors,
        };
        let formatted_errors = Error::format_errors(
            input.as_str(),
            path.file_name().and_then(|f| f.to_str()),
            errors,
        );
        insta::assert_snapshot!(formatted_errors);
    });
}

#[test]
fn test_valid() {
    insta::glob!("test_data/schema_definition/valid/*.graphql", |path| {
        let input = std::fs::read_to_string(path).unwrap();
        let executable_document: Result<DefinitionDocument, _> =
            DefinitionDocument::parse(input.as_str());
        assert!(executable_document.is_ok(), "Document had errors");
        let executable_document = executable_document.unwrap();
        let schema_definition = SchemaDefinition::try_from(&executable_document);
        assert!(schema_definition.is_ok(), "Document had errors");
    });
}
