use crate::ast::{FromTokens, ParseError, Tokens};
use crate::lexical_token::{Name, PunctuatorType};
use crate::Span;
use bluejay_core::{AbstractTypeReference, TypeReference as CoreTypeReference};

#[derive(Debug)]
pub enum TypeReference<'a> {
    NamedType {
        name: Name<'a>,
        bang_span: Option<Span>,
    },
    ListType {
        inner: Box<Self>,
        square_bracket_span: Span,
        bang_span: Option<Span>,
    },
}

impl<'a> AbstractTypeReference for TypeReference<'a> {
    fn as_ref(&self) -> CoreTypeReference<'_, Self> {
        match self {
            Self::NamedType { name, bang_span } => {
                CoreTypeReference::NamedType(name.as_ref(), bang_span.is_some())
            }
            Self::ListType {
                inner, bang_span, ..
            } => CoreTypeReference::ListType(inner.as_ref(), bang_span.is_some()),
        }
    }
}

impl<'a> FromTokens<'a> for TypeReference<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        if let Some(open_span) = tokens.next_if_punctuator(PunctuatorType::OpenSquareBracket) {
            let inner = Box::new(TypeReference::from_tokens(tokens)?);
            let close_span = tokens.expect_punctuator(PunctuatorType::CloseSquareBracket)?;
            let square_bracket_span = open_span.merge(&close_span);
            let bang_span = tokens.next_if_punctuator(PunctuatorType::Bang);
            Ok(Self::ListType {
                inner,
                square_bracket_span,
                bang_span,
            })
        } else if let Some(name) = tokens.next_if_name() {
            let bang_span = tokens.next_if_punctuator(PunctuatorType::Bang);
            Ok(Self::NamedType { name, bang_span })
        } else {
            Err(tokens.unexpected_token())
        }
    }
}
