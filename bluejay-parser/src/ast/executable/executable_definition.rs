use crate::ast::executable::{FragmentDefinition, OperationDefinition};
use crate::ast::{DepthLimiter, FromTokens, IsMatch, ParseError, Tokens};

#[derive(Debug)]
pub enum ExecutableDefinition<'a> {
    Operation(OperationDefinition<'a>),
    Fragment(FragmentDefinition<'a>),
}

impl<'a> FromTokens<'a> for ExecutableDefinition<'a> {
    #[inline]
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        // don't bump depth limiters because this is just a thin wrapper in the AST
        if OperationDefinition::is_match(tokens) {
            OperationDefinition::from_tokens(tokens, depth_limiter).map(Self::Operation)
        } else if FragmentDefinition::is_match(tokens) {
            FragmentDefinition::from_tokens(tokens, depth_limiter).map(Self::Fragment)
        } else {
            Err(tokens.unexpected_token())
        }
    }
}

impl<'a> IsMatch<'a> for ExecutableDefinition<'a> {
    #[inline]
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        OperationDefinition::is_match(tokens) || FragmentDefinition::is_match(tokens)
    }
}
