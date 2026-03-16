use crate::ast::{DepthLimiter, FromTokens, IsMatch, ParseError, Tokens};
use crate::{HasSpan, Span};

#[derive(Debug)]
pub struct OperationType {
    inner: bluejay_core::OperationType,
    span: Span,
}

impl HasSpan for OperationType {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl From<&OperationType> for bluejay_core::OperationType {
    fn from(value: &OperationType) -> Self {
        value.inner
    }
}

impl<'a> FromTokens<'a> for OperationType {
    #[inline]
    fn from_tokens(tokens: &mut impl Tokens<'a>, _: DepthLimiter) -> Result<Self, ParseError> {
        tokens.expect_name().and_then(|name| {
            match bluejay_core::OperationType::try_from(name.as_str()) {
                Ok(operation_type) => Ok(Self {
                    inner: operation_type,
                    span: name.into(),
                }),
                Err(_) => Err(ParseError::ExpectedOneOf {
                    span: name.into(),
                    values: bluejay_core::OperationType::POSSIBLE_VALUES,
                }),
            }
        })
    }
}

impl<'a> IsMatch<'a> for OperationType {
    #[inline]
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        bluejay_core::OperationType::POSSIBLE_VALUES
            .iter()
            .any(|value| tokens.peek_name_matches(0, value))
    }
}
