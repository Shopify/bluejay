use crate::ast::definition::EnumValueDefinition;
use crate::ast::{FromTokens, ParseError, Tokens};
use crate::lexical_token::PunctuatorType;
use crate::Span;
use bluejay_core::definition::EnumValueDefinitions as CoreEnumValueDefinitions;
use bluejay_core::AsIter;

#[derive(Debug)]
pub struct EnumValueDefinitions<'a> {
    enum_value_definitions: Vec<EnumValueDefinition<'a>>,
    _span: Span,
}

impl<'a> AsIter for EnumValueDefinitions<'a> {
    type Item = EnumValueDefinition<'a>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.enum_value_definitions.iter()
    }
}

impl<'a> CoreEnumValueDefinitions for EnumValueDefinitions<'a> {
    type EnumValueDefinition = EnumValueDefinition<'a>;
}

impl<'a> FromTokens<'a> for EnumValueDefinitions<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let open_span = tokens.expect_punctuator(PunctuatorType::OpenBrace)?;
        let mut enum_value_definitions = Vec::new();
        let close_span = loop {
            if let Some(close_span) = tokens.next_if_punctuator(PunctuatorType::CloseBrace) {
                break close_span;
            }
            enum_value_definitions.push(EnumValueDefinition::from_tokens(tokens)?);
        };
        let span = open_span.merge(&close_span);
        Ok(Self {
            enum_value_definitions,
            _span: span,
        })
    }
}
