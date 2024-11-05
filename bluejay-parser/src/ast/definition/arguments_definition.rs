use crate::ast::definition::{Context, InputValueDefinition};
use crate::ast::{DepthLimiter, FromTokens, IsMatch, ParseError, Tokens};
use crate::lexical_token::PunctuatorType;
use crate::Span;
use bluejay_core::definition::ArgumentsDefinition as CoreArgumentsDefinition;
use bluejay_core::AsIter;

#[derive(Debug)]
pub struct ArgumentsDefinition<'a, C: Context> {
    argument_definitions: Vec<InputValueDefinition<'a, C>>,
    _span: Span,
}

impl<'a, C: Context> AsIter for ArgumentsDefinition<'a, C> {
    type Item = InputValueDefinition<'a, C>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.argument_definitions.iter()
    }
}

impl<'a, C: Context> CoreArgumentsDefinition for ArgumentsDefinition<'a, C> {
    type ArgumentDefinition = InputValueDefinition<'a, C>;
}

impl<'a, C: Context> FromTokens<'a> for ArgumentsDefinition<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        let open_span = tokens.expect_punctuator(PunctuatorType::OpenRoundBracket)?;
        let mut argument_definitions: Vec<InputValueDefinition<C>> = Vec::new();
        let close_span = loop {
            argument_definitions.push(InputValueDefinition::from_tokens(
                tokens,
                depth_limiter.bump()?,
            )?);
            if let Some(close_span) = tokens.next_if_punctuator(PunctuatorType::CloseRoundBracket) {
                break close_span;
            }
        };
        let span = open_span.merge(&close_span);
        Ok(Self {
            argument_definitions,
            _span: span,
        })
    }
}

impl<'a, C: Context> IsMatch<'a> for ArgumentsDefinition<'a, C> {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        tokens.peek_punctuator_matches(0, PunctuatorType::OpenRoundBracket)
    }
}
