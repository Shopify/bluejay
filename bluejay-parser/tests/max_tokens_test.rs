use bluejay_parser::{
    ast::{executable::ExecutableDefinition, Parse, ParseDetails, ParseOptions},
    Error,
};

#[test]
fn test_token_counting() {
    let query = "query { user { id name } }";

    let options = ParseOptions {
        max_tokens: None,
        ..Default::default()
    };

    let result: Result<(ExecutableDefinition, ParseDetails), Vec<Error>> =
        ExecutableDefinition::parse_with_details_and_options(query, options);

    assert!(result.is_ok());
    let (_, details) = result.unwrap();

    assert_eq!(8, details.token_count);
}

#[test]
fn test_max_tokens_limit_exceeded() {
    let query = "query { user { id name email address phone } }";

    let options = ParseOptions {
        max_tokens: Some(3),
        ..Default::default()
    };

    let result: Result<(ExecutableDefinition, ParseDetails), Vec<Error>> =
        ExecutableDefinition::parse_with_details_and_options(query, options);

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

    let result: Result<(ExecutableDefinition, ParseDetails), Vec<Error>> =
        ExecutableDefinition::parse_with_details_and_options(query, options);

    assert!(result.is_ok());
    let (_, details) = result.unwrap();

    assert_eq!(8, details.token_count);
}

#[test]
fn test_max_tokens_limit_at_limit() {
    let query = "query { user { id name } }";

    let options = ParseOptions {
        max_tokens: Some(8),
        ..Default::default()
    };

    let result: Result<(ExecutableDefinition, ParseDetails), Vec<Error>> =
        ExecutableDefinition::parse_with_details_and_options(query, options);

    assert!(result.is_ok());
    let (_, details) = result.unwrap();

    assert_eq!(8, details.token_count);
}

#[test]
fn test_lexer_stops_after_max_tokens_and_prevents_stack_overflows() {
    // Query that would cause a stack overflow if fully parsed
    let deeply_nested = "{ a ".repeat(5000) + &"}".repeat(5000);

    let options = ParseOptions {
        max_tokens: Some(10),
        ..Default::default()
    };

    let result = ExecutableDefinition::parse_with_details_and_options(&deeply_nested, options);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert_eq!("Max tokens exceeded", errors[0].message());
}
