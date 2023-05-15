use crate::ast::definition::{Context, FieldDefinition};
use crate::ast::{FromTokens, ParseError, Tokens};
use crate::lexical_token::PunctuatorType;
use crate::Span;
use bluejay_core::definition::FieldsDefinition as CoreFieldsDefinition;
use bluejay_core::AsIter;

#[derive(Debug)]
pub struct FieldsDefinition<'a, C: Context> {
    field_definitions: Vec<FieldDefinition<'a, C>>,
    _span: Span,
}

impl<'a, C: Context> AsIter for FieldsDefinition<'a, C> {
    type Item = FieldDefinition<'a, C>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.field_definitions.iter()
    }
}

impl<'a, C: Context> CoreFieldsDefinition for FieldsDefinition<'a, C> {
    type FieldDefinition = FieldDefinition<'a, C>;
}

impl<'a, C: Context> FromTokens<'a> for FieldsDefinition<'a, C> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let open_span = tokens.expect_punctuator(PunctuatorType::OpenBrace)?;
        let mut field_definitions: Vec<FieldDefinition<'a, C>> =
            vec![FieldDefinition::__typename()];
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

impl<'a, C: Context> FieldsDefinition<'a, C> {
    pub(crate) fn add_query_root_fields(&mut self) {
        self.field_definitions.push(FieldDefinition::__schema());
        self.field_definitions.push(FieldDefinition::__type());
    }
}
