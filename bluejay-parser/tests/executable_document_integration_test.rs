use bluejay_parser::{ast::executable::ExecutableDocument, Error};

#[test]
fn test_error() {
    insta::glob!("test_data/executable_document/error/*.graphql", |path| {
        let input = std::fs::read_to_string(path).unwrap();
        let executable_document = ExecutableDocument::parse(input.as_str());
        assert!(
            executable_document.is_err(),
            "Document did not have any errors"
        );
        let errors = executable_document.unwrap_err();
        let formatted_errors = Error::format_errors(input.as_str(), errors);
        insta::assert_snapshot!(formatted_errors);
    });
}

#[test]
fn test_valid() {
    insta::glob!("test_data/executable_document/valid/*.graphql", |path| {
        let input = std::fs::read_to_string(path).unwrap();
        let executable_document = ExecutableDocument::parse(input.as_str());
        assert!(executable_document.is_ok(), "Document had errors");
    });
}
