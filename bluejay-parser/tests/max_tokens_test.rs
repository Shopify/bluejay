use bluejay_parser::{
    ast::{executable::ExecutableDefinition, Parse, ParseOptions, ParseResult},
    Error,
};

#[test]
fn test_token_counting() {
    let query = "query { user { id name } }";

    let options = ParseOptions {
        max_tokens: None,
        ..Default::default()
    };

    let result: Result<ParseResult<ExecutableDefinition>, Vec<Error>> =
        ExecutableDefinition::parse_with_options(query, options);

    assert!(result.is_ok());
    let parse_result = result.unwrap();

    assert_eq!(8, parse_result.token_count);
}

#[test]
fn test_max_tokens_limit_exceeded() {
    let query = "query { user { id name email address phone } }";

    let options = ParseOptions {
        max_tokens: Some(3),
        ..Default::default()
    };

    let result: Result<ParseResult<ExecutableDefinition>, Vec<Error>> =
        ExecutableDefinition::parse_with_options(query, options);

    assert!(result.is_err());
    let errors = result.unwrap_err();

    assert_eq!(1, errors.len());
    assert_eq!("Max tokens exceeded", errors[0].message());
}

#[test]
fn test_max_tokens_limit_not_exceeded() {
    let query = "query { user { id name } }";

    let options = ParseOptions {
        max_tokens: Some(50),
        ..Default::default()
    };

    let result: Result<ParseResult<ExecutableDefinition>, Vec<Error>> =
        ExecutableDefinition::parse_with_options(query, options);

    assert!(result.is_ok());
    let parse_result = result.unwrap();

    assert_eq!(8, parse_result.token_count);
}

#[test]
fn test_max_tokens_limit_at_limit() {
    let query = "query { user { id name } }";

    let options = ParseOptions {
        max_tokens: Some(8),
        ..Default::default()
    };

    let result: Result<ParseResult<ExecutableDefinition>, Vec<Error>> =
        ExecutableDefinition::parse_with_options(query, options);

    assert!(result.is_ok());
    let parse_result = result.unwrap();

    assert_eq!(8, parse_result.token_count);
}
