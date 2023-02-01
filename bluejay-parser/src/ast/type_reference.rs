use crate::ast::{FromTokens, IsMatch, ParseError, Tokens, TryFromTokens};
use crate::lexical_token::{Name, PunctuatorType};
use crate::Span;
use bluejay_core::{
    ListTypeReference as CoreListTypeReference, NamedTypeReference as CoreNamedTypeReference,
    TypeReference as CoreTypeReference,
};

pub type TypeReference<'a> = CoreTypeReference<NamedTypeReference<'a>, ListTypeReference<'a>>;

impl<'a> FromTokens<'a> for TypeReference<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        if let Some(ltr) = ListTypeReference::try_from_tokens(tokens) {
            ltr.map(Self::ListType)
        } else if let Some(ntr) = NamedTypeReference::try_from_tokens(tokens) {
            ntr.map(Self::NamedType)
        } else {
            Err(tokens.unexpected_token())
        }
    }
}

#[derive(Debug)]
pub struct NamedTypeReference<'a> {
    name: Name<'a>,
    bang_span: Option<Span>,
}

impl<'a> IsMatch<'a> for NamedTypeReference<'a> {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        tokens.peek_name(0).is_some()
    }
}

impl<'a> FromTokens<'a> for NamedTypeReference<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let name = tokens.expect_name()?;
        let bang_span = tokens.next_if_punctuator(PunctuatorType::Bang);
        Ok(Self { name, bang_span })
    }
}

impl<'a> CoreNamedTypeReference for NamedTypeReference<'a> {
    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn required(&self) -> bool {
        self.bang_span.is_some()
    }
}

#[derive(Debug)]
pub struct ListTypeReference<'a> {
    inner: Box<TypeReference<'a>>,
    _square_bracket_span: Span,
    bang_span: Option<Span>,
}

impl<'a> FromTokens<'a> for ListTypeReference<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let open_span = tokens.expect_punctuator(PunctuatorType::OpenSquareBracket)?;
        let inner = Box::new(TypeReference::from_tokens(tokens)?);
        let close_span = tokens.expect_punctuator(PunctuatorType::CloseSquareBracket)?;
        let square_bracket_span = open_span.merge(&close_span);
        let bang_span = tokens.next_if_punctuator(PunctuatorType::Bang);
        Ok(Self {
            inner,
            _square_bracket_span: square_bracket_span,
            bang_span,
        })
    }
}

impl<'a> IsMatch<'a> for ListTypeReference<'a> {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        tokens.peek_punctuator_matches(0, PunctuatorType::OpenSquareBracket)
    }
}

impl<'a> CoreListTypeReference for ListTypeReference<'a> {
    type NamedTypeReference = NamedTypeReference<'a>;

    fn inner(&self) -> &TypeReference<'a> {
        &self.inner
    }

    fn required(&self) -> bool {
        self.bang_span.is_some()
    }
}
