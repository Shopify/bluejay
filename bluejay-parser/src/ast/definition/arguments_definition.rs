use crate::ast::definition::InputValueDefinition;
use crate::ast::{FromTokens, IsMatch, ParseError, Tokens};
use crate::lexical_token::PunctuatorType;
use crate::Span;
use bluejay_core::definition::ArgumentsDefinition as CoreArgumentsDefinition;
use bluejay_core::AsIter;

#[derive(Debug)]
pub struct ArgumentsDefinition<'a> {
    argument_definitions: Vec<InputValueDefinition<'a>>,
    _span: Span,
}

impl<'a> AsIter for ArgumentsDefinition<'a> {
    type Item = InputValueDefinition<'a>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.argument_definitions.iter()
    }
}

impl<'a> CoreArgumentsDefinition for ArgumentsDefinition<'a> {
    type ArgumentDefinition = InputValueDefinition<'a>;
}

impl<'a> FromTokens<'a> for ArgumentsDefinition<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let open_span = tokens.expect_punctuator(PunctuatorType::OpenRoundBracket)?;
        let mut argument_definitions: Vec<InputValueDefinition> = Vec::new();
        let close_span = loop {
            argument_definitions.push(InputValueDefinition::from_tokens(tokens)?);
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

impl<'a> IsMatch<'a> for ArgumentsDefinition<'a> {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        tokens.peek_punctuator_matches(0, PunctuatorType::OpenRoundBracket)
    }
}
