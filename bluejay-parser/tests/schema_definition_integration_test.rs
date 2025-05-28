use bluejay_parser::{
    ast::{
        definition::{DefaultContext, DefinitionDocument, SchemaDefinition},
        Parse,
    },
    Error,
};

#[test]
fn test_error() {
    insta::glob!("test_data/schema_definition/error/*.graphql", |path| {
        let input = std::fs::read_to_string(path).unwrap();
        let definition_document: Result<_, _> =
            DefinitionDocument::<DefaultContext>::parse(input.as_str());
        let errors = match definition_document {
            Ok(parse_result) => match SchemaDefinition::try_from(&parse_result.parsed) {
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
        let definition_document: Result<_, _> =
            DefinitionDocument::<DefaultContext>::parse(input.as_str());
        assert!(definition_document.is_ok(), "Document had errors");
        let definition_document = definition_document.unwrap();
        let schema_definition = SchemaDefinition::try_from(&definition_document.parsed);
        assert!(schema_definition.is_ok(), "Document had errors");
    });
}
