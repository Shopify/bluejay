use crate::ast::{FromTokens, IsMatch, ParseError, Tokens};
use crate::lexical_token::HasSpan;

impl<'a> FromTokens<'a> for bluejay_core::OperationType {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        tokens.expect_name().and_then(|name| {
            bluejay_core::OperationType::try_from(name.as_str()).map_err(|_| {
                ParseError::ExpectedOneOf {
                    span: name.span(),
                    values: bluejay_core::OperationType::POSSIBLE_VALUES,
                }
            })
        })
    }
}

impl<'a> IsMatch<'a> for bluejay_core::OperationType {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        Self::POSSIBLE_VALUES
            .iter()
            .any(|value| tokens.peek_name_matches(0, value))
    }
}
