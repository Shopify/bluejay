use crate::ast::{FromTokens, LexerTokens};
use crate::lexer::LogosLexer;
use crate::Error;

pub trait Parse<'a>: FromTokens<'a> {
    fn parse(s: &'a str) -> Result<Self, Vec<Error>>;
}

impl<'a, T: FromTokens<'a>> Parse<'a> for T {
    fn parse(s: &'a str) -> Result<Self, Vec<Error>> {
        let lexer = LogosLexer::new(s);
        let mut tokens = LexerTokens::new(lexer);

        let result = T::from_tokens(&mut tokens);

        if tokens.errors.is_empty() {
            result.map_err(|err| vec![err.into()])
        } else {
            Err(tokens.errors.into_iter().map(Into::into).collect())
        }
    }
}
