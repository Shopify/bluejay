use crate::ast::{FromTokens, ParseError, Tokens};
use crate::lexical_token::{Name, PunctuatorType};
use crate::{HasSpan, Span};
use bluejay_core::{AbstractTypeReference, TypeReference as CoreTypeReference};
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub enum TypeReference<'a> {
    NamedType {
        name: Name<'a>,
        is_required: bool,
        span: Span,
    },
    ListType {
        inner: Box<Self>,
        is_required: bool,
        span: Span,
    },
}

impl<'a> AbstractTypeReference for TypeReference<'a> {
    fn as_ref(&self) -> CoreTypeReference<'_, Self> {
        match self {
            Self::NamedType {
                name, is_required, ..
            } => CoreTypeReference::NamedType(name.as_ref(), *is_required),
            Self::ListType {
                inner, is_required, ..
            } => CoreTypeReference::ListType(inner.as_ref(), *is_required),
        }
    }
}

impl<'a> FromTokens<'a> for TypeReference<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        if let Some(open_span) = tokens.next_if_punctuator(PunctuatorType::OpenSquareBracket) {
            let inner = Box::new(TypeReference::from_tokens(tokens)?);
            let close_span = tokens.expect_punctuator(PunctuatorType::CloseSquareBracket)?;
            let bang_span = tokens.next_if_punctuator(PunctuatorType::Bang);
            let is_required = bang_span.is_some();
            let close_span = bang_span.unwrap_or(close_span);
            let span = open_span.merge(&close_span);
            Ok(Self::ListType {
                inner,
                is_required,
                span,
            })
        } else if let Some(name) = tokens.next_if_name() {
            let bang_span = tokens.next_if_punctuator(PunctuatorType::Bang);
            let is_required = bang_span.is_some();
            let span = bang_span
                .map(|bang_span| name.span().merge(&bang_span))
                .unwrap_or(name.span().clone());
            Ok(Self::NamedType {
                name,
                is_required,
                span,
            })
        } else {
            Err(tokens.unexpected_token())
        }
    }
}

impl<'a> HasSpan for TypeReference<'a> {
    fn span(&self) -> &Span {
        match self {
            Self::NamedType { span, .. } => span,
            Self::ListType { span, .. } => span,
        }
    }
}

impl<'a> Hash for TypeReference<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.span().hash(state);
    }
}

impl<'a> PartialEq for TypeReference<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.span() == other.span()
    }
}

impl<'a> Eq for TypeReference<'a> {}

impl<'a> Ord for TypeReference<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.span().cmp(other.span())
    }
}

impl<'a> PartialOrd for TypeReference<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
