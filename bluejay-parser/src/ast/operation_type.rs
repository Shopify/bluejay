use crate::ast::{FromTokens, IsMatch, ParseError, Tokens};
use crate::{HasSpan, Span};

#[derive(Debug)]
pub struct OperationType {
    inner: bluejay_core::OperationType,
    span: Span,
}

impl HasSpan for OperationType {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl From<&OperationType> for bluejay_core::OperationType {
    fn from(value: &OperationType) -> Self {
        value.inner
    }
}

impl<'a> FromTokens<'a> for OperationType {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        tokens.expect_name().and_then(|name| {
            bluejay_core::OperationType::try_from(name.as_str())
                .map_err(|_| ParseError::ExpectedOneOf {
                    span: name.span(),
                    values: bluejay_core::OperationType::POSSIBLE_VALUES,
                })
                .map(|operation_type| Self {
                    inner: operation_type,
                    span: name.span(),
                })
        })
    }
}

impl<'a> IsMatch<'a> for OperationType {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        bluejay_core::OperationType::POSSIBLE_VALUES
            .iter()
            .any(|value| tokens.peek_name_matches(0, value))
    }
}
