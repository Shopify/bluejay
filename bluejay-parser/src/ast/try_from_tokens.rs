use crate::ast::{DepthLimiter, ParseError, Tokens};

pub trait TryFromTokens<'a>: Sized {
    fn try_from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Option<Result<Self, ParseError>>;
}
