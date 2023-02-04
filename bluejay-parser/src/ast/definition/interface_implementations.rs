use crate::ast::definition::InterfaceImplementation;
use crate::ast::{FromTokens, IsMatch, ParseError, Tokens};
use crate::lexical_token::PunctuatorType;
use bluejay_core::definition::InterfaceImplementations as CoreInterfaceImplementations;
use bluejay_core::AsIter;

#[derive(Debug)]
pub struct InterfaceImplementations<'a> {
    interface_implementations: Vec<InterfaceImplementation<'a>>,
}

impl<'a> AsIter for InterfaceImplementations<'a> {
    type Item = InterfaceImplementation<'a>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.interface_implementations.iter()
    }
}

impl<'a> CoreInterfaceImplementations for InterfaceImplementations<'a> {
    type InterfaceImplementation = InterfaceImplementation<'a>;
}

impl<'a> InterfaceImplementations<'a> {
    const IMPLEMENTS_IDENTIFIER: &'static str = "implements";
}

impl<'a> FromTokens<'a> for InterfaceImplementations<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        tokens.expect_name_value(Self::IMPLEMENTS_IDENTIFIER)?;
        tokens.next_if_punctuator(PunctuatorType::Ampersand);
        let mut interface_implementations = vec![InterfaceImplementation::from_tokens(tokens)?];
        while tokens
            .next_if_punctuator(PunctuatorType::Ampersand)
            .is_some()
        {
            interface_implementations.push(InterfaceImplementation::from_tokens(tokens)?);
        }
        Ok(Self {
            interface_implementations,
        })
    }
}

impl<'a> IsMatch<'a> for InterfaceImplementations<'a> {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        tokens.peek_name_matches(0, Self::IMPLEMENTS_IDENTIFIER)
    }
}
