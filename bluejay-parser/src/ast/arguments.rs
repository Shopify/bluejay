use crate::ast::{Argument, DepthLimiter, FromTokens, IsMatch, ParseError, Tokens};
use crate::lexical_token::PunctuatorType;
use crate::{HasSpan, Span};
use bluejay_core::AsIter;

#[derive(Debug)]
pub struct Arguments<'a, const CONST: bool> {
    arguments: Vec<Argument<'a, CONST>>,
    span: Span,
}

pub type VariableArguments<'a> = Arguments<'a, false>;

impl<'a, const CONST: bool> FromTokens<'a> for Arguments<'a, CONST> {
    #[inline]
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        let open_span = tokens.expect_punctuator(PunctuatorType::OpenRoundBracket)?;
        let mut arguments: Vec<Argument<CONST>> = Vec::new();
        let close_span = loop {
            arguments.push(Argument::from_tokens(tokens, depth_limiter.bump()?)?);
            if let Some(close_span) = tokens.next_if_punctuator(PunctuatorType::CloseRoundBracket) {
                break close_span;
            }
        };
        let span = open_span.merge(&close_span);
        Ok(Self { arguments, span })
    }
}

impl<'a, const CONST: bool> IsMatch<'a> for Arguments<'a, CONST> {
    #[inline]
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        tokens.peek_punctuator_matches(0, PunctuatorType::OpenRoundBracket)
    }
}

impl<const CONST: bool> HasSpan for Arguments<'_, CONST> {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl<'a, const CONST: bool> bluejay_core::Arguments<CONST> for Arguments<'a, CONST> {
    type Argument = Argument<'a, CONST>;
}

impl<'a, const CONST: bool> AsIter for Arguments<'a, CONST> {
    type Item = Argument<'a, CONST>;
    type Iterator<'b>
        = std::slice::Iter<'b, Self::Item>
    where
        'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.arguments.iter()
    }
}
