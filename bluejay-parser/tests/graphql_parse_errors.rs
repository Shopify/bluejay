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

    assert!(document.is_err());
    if let Err(document_errors) = document {
        let graphql_errors = Error::into_graphql_error(source, document_errors);
        let expected: Vec<GraphQLError> = vec![GraphQLError {
            message: "Parse error".to_string(),
            locations: vec![Location { line: 2, col: 15 }],
        }];
        assert_eq!(expected, graphql_errors);
    }
}
