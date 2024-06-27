use crate::ast::executable::VariableDefinition;
use crate::ast::{FromTokens, IsMatch, ParseError, Tokens};
use crate::lexical_token::PunctuatorType;
use crate::Span;
use bluejay_core::AsIter;

#[derive(Debug)]
pub struct VariableDefinitions<'a> {
    variable_definitions: Vec<VariableDefinition<'a>>,
    _span: Span,
}

impl<'a> FromTokens<'a> for VariableDefinitions<'a> {
    #[inline]
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let open_span = tokens.expect_punctuator(PunctuatorType::OpenRoundBracket)?;
        let mut variable_definitions: Vec<VariableDefinition> = Vec::new();
        let close_span = loop {
            variable_definitions.push(VariableDefinition::from_tokens(tokens)?);
            if let Some(close_span) = tokens.next_if_punctuator(PunctuatorType::CloseRoundBracket) {
                break close_span;
            }
        };
        let span = open_span.merge(&close_span);
        Ok(Self {
            variable_definitions,
            _span: span,
        })
    }
}

impl<'a> IsMatch<'a> for VariableDefinitions<'a> {
    #[inline]
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        tokens.peek_punctuator_matches(0, PunctuatorType::OpenRoundBracket)
    }
}

impl<'a> bluejay_core::executable::VariableDefinitions for VariableDefinitions<'a> {
    type VariableDefinition = VariableDefinition<'a>;
}

impl<'a> AsIter for VariableDefinitions<'a> {
    type Item = VariableDefinition<'a>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.variable_definitions.iter()
    }
}
