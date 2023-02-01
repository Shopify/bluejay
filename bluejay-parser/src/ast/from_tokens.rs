use crate::ast::{ParseError, Tokens};

pub trait FromTokens<'a>: Sized {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError>;
}
