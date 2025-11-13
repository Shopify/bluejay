use bluejay_parser::{
    ast::{executable::ExecutableDocument, Parse, ParseOptions},
    Error,
};

#[test]
fn test_error() {
    insta::glob!("test_data/executable_document/error/*.graphql", |path| {
        let input = std::fs::read_to_string(path).unwrap();
        let executable_document = ExecutableDocument::parse(input.as_str());
        assert!(
            executable_document.result.is_err(),
            "Document did not have any errors"
        );
        let errors = executable_document.result.unwrap_err();
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
    insta::glob!("test_data/executable_document/valid/*.graphql", |path| {
        let input = std::fs::read_to_string(path).unwrap();
        let executable_document = ExecutableDocument::parse(input.as_str());
        assert!(executable_document.result.is_ok(), "Document had errors");
    });
}

#[test]
fn test_graphql_ruby_valid() {
    insta::glob!(
        "test_data/executable_document/graphql_ruby_valid/*.graphql",
        |path| {
            let input = std::fs::read_to_string(path).unwrap();
            let executable_document = ExecutableDocument::parse_with_options(
                input.as_str(),
                ParseOptions {
                    graphql_ruby_compatibility: true,
                    ..Default::default()
                },
            );
            assert!(executable_document.result.is_ok(), "Document had errors");
        }
    );
}
