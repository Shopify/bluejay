use crate::ast::{DepthLimiter, FromTokens, IsMatch, ParseError, Tokens};
use crate::lexical_token::Name;

#[derive(Debug)]
pub struct TypeCondition<'a> {
    named_type: Name<'a>,
}

impl<'a> FromTokens<'a> for TypeCondition<'a> {
    #[inline]
    fn from_tokens(tokens: &mut impl Tokens<'a>, _: DepthLimiter) -> Result<Self, ParseError> {
        tokens.expect_name_value(Self::ON)?;
        let named_type = tokens.expect_name()?;
        Ok(Self { named_type })
    }
}

impl<'a> IsMatch<'a> for TypeCondition<'a> {
    #[inline]
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        tokens.peek_name_matches(0, Self::ON)
    }
}

impl TypeCondition<'_> {
    pub(crate) const ON: &'static str = "on";

    pub fn named_type(&self) -> &Name {
        &self.named_type
    }
}
