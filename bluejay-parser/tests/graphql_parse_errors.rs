use std::borrow::Cow;

use bluejay_parser::{
    ast::{executable::ExecutableDocument, Parse},
    error::{GraphQLError, Location},
    Error,
};

#[test]
fn test_parser_errors() {
    let source = r#"{
        field()
    }"#;
    let document = ExecutableDocument::parse(source);

    assert!(document.result.is_err());
    let document_errors = document.result.unwrap_err();
    let graphql_errors = Error::into_graphql_errors(source, document_errors);
    let expected: Vec<GraphQLError> = vec![GraphQLError {
        message: Cow::from("Expected a name"),
        locations: vec![Location { line: 2, col: 15 }],
    }];
    assert_eq!(expected, graphql_errors);
}
