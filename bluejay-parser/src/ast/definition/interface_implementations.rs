use crate::ast::definition::{Context, InterfaceImplementation};
use crate::ast::{DepthLimiter, FromTokens, IsMatch, ParseError, Tokens};
use crate::lexical_token::PunctuatorType;
use bluejay_core::definition::InterfaceImplementations as CoreInterfaceImplementations;
use bluejay_core::AsIter;

#[derive(Debug)]
pub struct InterfaceImplementations<'a, C: Context + 'a> {
    interface_implementations: Vec<InterfaceImplementation<'a, C>>,
}

impl<'a, C: Context + 'a> AsIter for InterfaceImplementations<'a, C> {
    type Item = InterfaceImplementation<'a, C>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.interface_implementations.iter()
    }
}

impl<'a, C: Context + 'a> CoreInterfaceImplementations for InterfaceImplementations<'a, C> {
    type InterfaceImplementation = InterfaceImplementation<'a, C>;
}

impl<'a, C: Context + 'a> InterfaceImplementations<'a, C> {
    const IMPLEMENTS_IDENTIFIER: &'static str = "implements";
}

impl<'a, C: Context + 'a> FromTokens<'a> for InterfaceImplementations<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        tokens.expect_name_value(Self::IMPLEMENTS_IDENTIFIER)?;
        tokens.next_if_punctuator(PunctuatorType::Ampersand);
        let mut interface_implementations = vec![InterfaceImplementation::from_tokens(
            tokens,
            depth_limiter.bump()?,
        )?];
        while tokens
            .next_if_punctuator(PunctuatorType::Ampersand)
            .is_some()
        {
            interface_implementations.push(InterfaceImplementation::from_tokens(
                tokens,
                depth_limiter.bump()?,
            )?);
        }
        Ok(Self {
            interface_implementations,
        })
    }
}

impl<'a, C: Context + 'a> IsMatch<'a> for InterfaceImplementations<'a, C> {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        tokens.peek_name_matches(0, Self::IMPLEMENTS_IDENTIFIER)
    }
}
