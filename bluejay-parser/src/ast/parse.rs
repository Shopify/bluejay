use crate::ast::{FromTokens, LexerTokens, Tokens};
use crate::lexer::LogosLexer;
use crate::Error;

#[derive(Default)]
pub struct ParseOptions {
    pub graphql_ruby_compatibility: bool,
}

pub trait Parse<'a>: Sized {
    fn parse(s: &'a str) -> Result<Self, Vec<Error>> {
        Self::parse_with_options(s, Default::default())
    }

    fn parse_with_options(s: &'a str, options: ParseOptions) -> Result<Self, Vec<Error>> {
        let lexer =
            LogosLexer::new(s).with_graphql_ruby_compatibility(options.graphql_ruby_compatibility);
        let tokens = LexerTokens::new(lexer);

        Self::parse_from_tokens(tokens)
    }

    fn parse_from_tokens(tokens: impl Tokens<'a>) -> Result<Self, Vec<Error>>;
}

impl<'a, T: FromTokens<'a>> Parse<'a> for T {
    fn parse_from_tokens(mut tokens: impl Tokens<'a>) -> Result<Self, Vec<Error>> {
        let result = T::from_tokens(&mut tokens);

        let errors = tokens.into_errors();

        if errors.is_empty() {
            result.map_err(|err| vec![err.into()])
        } else {
            Err(errors.into_iter().map(Into::into).collect())
        }
    }
}
