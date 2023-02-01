use crate::ast::{FromTokens, ParseError, Tokens, TryFromTokens};

pub trait IsMatch<'a> {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool;
}

impl<'a, T: FromTokens<'a> + IsMatch<'a>> TryFromTokens<'a> for T {
    fn try_from_tokens(tokens: &mut impl Tokens<'a>) -> Option<Result<Self, ParseError>> {
        Self::is_match(tokens).then(|| Self::from_tokens(tokens))
    }
}
