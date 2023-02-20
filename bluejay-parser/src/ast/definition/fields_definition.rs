use crate::ast::definition::FieldDefinition;
use crate::ast::{FromTokens, ParseError, Tokens};
use crate::lexical_token::PunctuatorType;
use crate::Span;
use bluejay_core::definition::FieldsDefinition as CoreFieldsDefinition;
use bluejay_core::AsIter;

#[derive(Debug)]
pub struct FieldsDefinition<'a> {
    field_definitions: Vec<FieldDefinition<'a>>,
    _span: Span,
}

impl<'a> AsIter for FieldsDefinition<'a> {
    type Item = FieldDefinition<'a>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.field_definitions.iter()
    }
}

impl<'a> CoreFieldsDefinition for FieldsDefinition<'a> {
    type FieldDefinition = FieldDefinition<'a>;
}

impl<'a> FromTokens<'a> for FieldsDefinition<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let open_span = tokens.expect_punctuator(PunctuatorType::OpenBrace)?;
        let mut field_definitions: Vec<FieldDefinition> = vec![FieldDefinition::typename()];
        let close_span = loop {
            field_definitions.push(FieldDefinition::from_tokens(tokens)?);
            if let Some(close_span) = tokens.next_if_punctuator(PunctuatorType::CloseBrace) {
                break close_span;
            }
        };
        let span = open_span.merge(&close_span);
        Ok(Self {
            field_definitions,
            _span: span,
        })
    }
}
