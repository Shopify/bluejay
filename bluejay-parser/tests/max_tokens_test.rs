use bluejay_parser::ast::{executable::ExecutableDefinition, Parse, ParseDetails, ParseOptions};

#[test]
fn test_token_counting() {
    let query = "query { user { id name } }";

    let options = ParseOptions {
        max_tokens: None,
        ..Default::default()
    };

    let result: ParseDetails<ExecutableDefinition> =
        ExecutableDefinition::parse_with_options(query, options);

    assert!(result.result.is_ok());
    assert_eq!(8, result.token_count);
}

#[test]
fn test_max_tokens_limit_exceeded() {
    let query = "query { user { id name email address phone } }";

    let options = ParseOptions {
        max_tokens: Some(3),
        ..Default::default()
    };

    let result: ParseDetails<ExecutableDefinition> =
        ExecutableDefinition::parse_with_options(query, options);

    assert!(result.result.is_err());
    let errors = result.result.unwrap_err();

    assert_eq!(1, errors.len());
    assert_eq!("Max tokens exceeded", errors[0].message());
    assert_eq!(4, result.token_count);
}

#[test]
fn test_max_tokens_limit_not_exceeded() {
    let query = "query { user { id name } }";

    let options = ParseOptions {
        max_tokens: Some(50),
        ..Default::default()
    };

    let result: ParseDetails<ExecutableDefinition> =
        ExecutableDefinition::parse_with_options(query, options);

    assert!(result.result.is_ok());
    assert_eq!(8, result.token_count);
}

#[test]
fn test_max_tokens_limit_at_limit() {
    let query = "query { user { id name } }";

    let options = ParseOptions {
        max_tokens: Some(8),
        ..Default::default()
    };

    let result: ParseDetails<ExecutableDefinition> =
        ExecutableDefinition::parse_with_options(query, options);

    assert!(result.result.is_ok());
    assert_eq!(8, result.token_count);
}

#[test]
fn test_lexer_stops_after_max_tokens_and_prevents_stack_overflows() {
    // Query that would cause a stack overflow if fully parsed
    let deeply_nested = "{ a ".repeat(5000) + &"}".repeat(5000);

    let options = ParseOptions {
        max_tokens: Some(10),
        ..Default::default()
    };

    let result = ExecutableDefinition::parse_with_options(&deeply_nested, options);
    assert!(result.result.is_err());

    let errors = result.result.unwrap_err();
    assert_eq!("Max tokens exceeded", errors[0].message());
    assert_eq!(11, result.token_count);
}
