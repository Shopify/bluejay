use crate::ast::{FromTokens, ParseError, Tokens};
use crate::lexical_token::{Name, PunctuatorType};
use crate::{HasSpan, Span};
use bluejay_core::{
    executable::{VariableType as CoreVariableType, VariableTypeReference},
    Indexable,
};

#[derive(Debug)]
pub enum VariableType<'a> {
    Named {
        name: Name<'a>,
        is_required: bool,
        span: Span,
    },
    List {
        inner: Box<Self>,
        is_required: bool,
        span: Span,
    },
}

impl<'a> Indexable for VariableType<'a> {
    type Id = Span;

    fn id(&self) -> &Self::Id {
        self.span()
    }
}

impl<'a> CoreVariableType for VariableType<'a> {
    fn as_ref(&self) -> VariableTypeReference<'_, Self> {
        match self {
            Self::Named {
                name, is_required, ..
            } => VariableTypeReference::Named(name.as_ref(), *is_required),
            Self::List {
                inner, is_required, ..
            } => VariableTypeReference::List(inner.as_ref(), *is_required),
        }
    }
}

impl<'a> FromTokens<'a> for VariableType<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        if let Some(open_span) = tokens.next_if_punctuator(PunctuatorType::OpenSquareBracket) {
            let inner = Box::new(VariableType::from_tokens(tokens)?);
            let close_span = tokens.expect_punctuator(PunctuatorType::CloseSquareBracket)?;
            let bang_span = tokens.next_if_punctuator(PunctuatorType::Bang);
            let is_required = bang_span.is_some();
            let close_span = bang_span.unwrap_or(close_span);
            let span = open_span.merge(&close_span);
            Ok(Self::List {
                inner,
                is_required,
                span,
            })
        } else if let Some(name) = tokens.next_if_name() {
            let bang_span = tokens.next_if_punctuator(PunctuatorType::Bang);
            let is_required = bang_span.is_some();
            let span = bang_span
                .map(|bang_span| name.span().merge(&bang_span))
                .unwrap_or_else(|| name.span().clone());
            Ok(Self::Named {
                name,
                is_required,
                span,
            })
        } else {
            Err(tokens.unexpected_token())
        }
    }
}

impl<'a> HasSpan for VariableType<'a> {
    fn span(&self) -> &Span {
        match self {
            Self::Named { span, .. } => span,
            Self::List { span, .. } => span,
        }
    }
}
