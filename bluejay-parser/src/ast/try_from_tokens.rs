use crate::ast::{Tokens, ParseError};

pub trait TryFromTokens<'a>: Sized {
    fn try_from_tokens(tokens: &mut impl Tokens<'a>) -> Option<Result<Self, ParseError>>;
}
