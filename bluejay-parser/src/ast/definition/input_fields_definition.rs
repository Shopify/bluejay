use crate::ast::definition::{Context, InputValueDefinition};
use crate::ast::{DepthLimiter, FromTokens, ParseError, Tokens};
use crate::lexical_token::PunctuatorType;
use crate::Span;
use bluejay_core::definition::InputFieldsDefinition as CoreInputFieldsDefinition;
use bluejay_core::AsIter;

#[derive(Debug)]
pub struct InputFieldsDefinition<'a, C: Context> {
    input_field_definitions: Vec<InputValueDefinition<'a, C>>,
    _span: Span,
}

impl<'a, C: Context> AsIter for InputFieldsDefinition<'a, C> {
    type Item = InputValueDefinition<'a, C>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.input_field_definitions.iter()
    }
}

impl<'a, C: Context> CoreInputFieldsDefinition for InputFieldsDefinition<'a, C> {
    type InputValueDefinition = InputValueDefinition<'a, C>;
}

impl<'a, C: Context> FromTokens<'a> for InputFieldsDefinition<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        let open_span = tokens.expect_punctuator(PunctuatorType::OpenBrace)?;
        let mut input_field_definitions: Vec<InputValueDefinition<'a, C>> = Vec::new();
        let close_span = loop {
            input_field_definitions.push(InputValueDefinition::from_tokens(
                tokens,
                depth_limiter.bump()?,
            )?);
            if let Some(close_span) = tokens.next_if_punctuator(PunctuatorType::CloseBrace) {
                break close_span;
            }
        };
        let span = open_span.merge(&close_span);
        Ok(Self {
            input_field_definitions,
            _span: span,
        })
    }
}
