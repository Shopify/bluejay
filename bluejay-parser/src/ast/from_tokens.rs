use crate::ast::{DepthLimiter, ParseError, Tokens};

pub trait FromTokens<'a>: Sized {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError>;
}
