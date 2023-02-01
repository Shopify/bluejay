use crate::ast::{ParseError, Tokens};

pub trait TryFromTokens<'a>: Sized {
    fn try_from_tokens(tokens: &mut impl Tokens<'a>) -> Option<Result<Self, ParseError>>;
}
