use crate::ast::executable::Selection;
use crate::ast::{FromTokens, IsMatch, ParseError, Tokens};
use crate::lexical_token::PunctuatorType;
use crate::{HasSpan, Span};
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct SelectionSet<'a> {
    selections: Vec<Selection<'a>>,
    span: Span,
}

impl<'a> FromTokens<'a> for SelectionSet<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let open_span = tokens.expect_punctuator(PunctuatorType::OpenBrace)?;
        let mut selections: Vec<Selection> = Vec::new();
        let close_span = loop {
            selections.push(Selection::from_tokens(tokens)?);
            if let Some(close_span) = tokens.next_if_punctuator(PunctuatorType::CloseBrace) {
                break close_span;
            }
        };
        let span = open_span.merge(&close_span);
        Ok(Self { selections, span })
    }
}

impl<'a> IsMatch<'a> for SelectionSet<'a> {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        tokens.peek_punctuator_matches(0, PunctuatorType::OpenBrace)
    }
}

impl<'a> bluejay_core::executable::SelectionSet for SelectionSet<'a> {
    type Selection = Selection<'a>;
}

impl<'a> AsRef<[Selection<'a>]> for SelectionSet<'a> {
    fn as_ref(&self) -> &[Selection<'a>] {
        &self.selections
    }
}

impl<'a> HasSpan for SelectionSet<'a> {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl<'a> Hash for SelectionSet<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.span.hash(state);
    }
}

impl<'a> PartialEq for SelectionSet<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.span == other.span
    }
}

impl<'a> Eq for SelectionSet<'a> {}

impl<'a> Ord for SelectionSet<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.span.cmp(&other.span)
    }
}

impl<'a> PartialOrd for SelectionSet<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
