use crate::ast::{depth_limiter::DEFAULT_MAX_DEPTH, DepthLimiter, FromTokens, LexerTokens, Tokens};
use crate::lexer::LogosLexer;
use crate::Error;

pub struct ParseOptions {
    pub graphql_ruby_compatibility: bool,
    pub max_depth: usize,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            graphql_ruby_compatibility: false,
            max_depth: DEFAULT_MAX_DEPTH,
        }
    }
}

pub trait Parse<'a>: Sized {
    #[inline]
    fn parse(s: &'a str) -> Result<Self, Vec<Error>> {
        Self::parse_with_options(s, Default::default())
    }

    #[inline]
    fn parse_with_options(s: &'a str, options: ParseOptions) -> Result<Self, Vec<Error>> {
        let lexer =
            LogosLexer::new(s).with_graphql_ruby_compatibility(options.graphql_ruby_compatibility);
        let tokens = LexerTokens::new(lexer);

        Self::parse_from_tokens(tokens, options.max_depth)
    }

    fn parse_from_tokens(tokens: impl Tokens<'a>, max_depth: usize) -> Result<Self, Vec<Error>>;
}

impl<'a, T: FromTokens<'a>> Parse<'a> for T {
    #[inline]
    fn parse_from_tokens(
        mut tokens: impl Tokens<'a>,
        max_depth: usize,
    ) -> Result<Self, Vec<Error>> {
        let result = T::from_tokens(&mut tokens, DepthLimiter::new(max_depth));

        let errors = tokens.into_errors();

        if errors.is_empty() {
            result.map_err(|err| vec![err.into()])
        } else {
            Err(errors.into_iter().map(Into::into).collect())
        }
    }
}
