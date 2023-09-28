use crate::ast::definition::{Context, EnumValueDefinition};
use crate::ast::{FromTokens, ParseError, Tokens};
use crate::lexical_token::PunctuatorType;
use crate::Span;
use bluejay_core::definition::EnumValueDefinitions as CoreEnumValueDefinitions;
use bluejay_core::AsIter;

#[derive(Debug)]
pub struct EnumValueDefinitions<'a, C: Context> {
    enum_value_definitions: Vec<EnumValueDefinition<'a, C>>,
    _span: Span,
}

impl<'a, C: Context> AsIter for EnumValueDefinitions<'a, C> {
    type Item = EnumValueDefinition<'a, C>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.enum_value_definitions.iter()
    }
}

impl<'a, C: Context> CoreEnumValueDefinitions for EnumValueDefinitions<'a, C> {
    type EnumValueDefinition = EnumValueDefinition<'a, C>;
}

impl<'a, C: Context> FromTokens<'a> for EnumValueDefinitions<'a, C> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let open_span = tokens.expect_punctuator(PunctuatorType::OpenBrace)?;
        let mut enum_value_definitions = Vec::new();
        let close_span = loop {
            enum_value_definitions.push(EnumValueDefinition::from_tokens(tokens)?);
            if let Some(close_span) = tokens.next_if_punctuator(PunctuatorType::CloseBrace) {
                break close_span;
            }
        };
        let span = open_span.merge(&close_span);
        Ok(Self {
            enum_value_definitions,
            _span: span,
        })
    }
}
