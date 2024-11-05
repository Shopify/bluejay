use crate::ast::executable::Selection;
use crate::ast::{DepthLimiter, FromTokens, IsMatch, ParseError, Tokens};
use crate::lexical_token::PunctuatorType;
use crate::{HasSpan, Span};
use bluejay_core::AsIter;

#[derive(Debug)]
pub struct SelectionSet<'a> {
    selections: Vec<Selection<'a>>,
    span: Span,
}

impl<'a> FromTokens<'a> for SelectionSet<'a> {
    #[inline]
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        let open_span = tokens.expect_punctuator(PunctuatorType::OpenBrace)?;
        let mut selections: Vec<Selection> = Vec::new();
        let close_span = loop {
            selections.push(Selection::from_tokens(tokens, depth_limiter.bump()?)?);
            if let Some(close_span) = tokens.next_if_punctuator(PunctuatorType::CloseBrace) {
                break close_span;
            }
        };
        let span = open_span.merge(&close_span);
        Ok(Self { selections, span })
    }
}

impl<'a> IsMatch<'a> for SelectionSet<'a> {
    #[inline]
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        tokens.peek_punctuator_matches(0, PunctuatorType::OpenBrace)
    }
}

impl<'a> bluejay_core::Indexable for SelectionSet<'a> {
    type Id = Span;

    fn id(&self) -> &Self::Id {
        &self.span
    }
}

impl<'a> bluejay_core::executable::SelectionSet for SelectionSet<'a> {
    type Selection = Selection<'a>;
}

impl<'a> AsIter for SelectionSet<'a> {
    type Item = Selection<'a>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.selections.iter()
    }

    fn len(&self) -> usize {
        self.selections.len()
    }
}

impl<'a> HasSpan for SelectionSet<'a> {
    fn span(&self) -> &Span {
        &self.span
    }
}
