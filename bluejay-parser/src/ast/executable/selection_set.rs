use crate::ast::executable::Selection;
use crate::ast::{ParseError, Tokens, FromTokens, IsMatch};
use crate::lexical_token::PunctuatorType;

#[derive(Debug)]
pub struct SelectionSet<'a> {
    selections: Vec<Selection<'a>>,
}

impl<'a> FromTokens<'a> for SelectionSet<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        tokens.expect_punctuator(PunctuatorType::OpenBrace)?;
        let mut selections: Vec<Selection> = Vec::new();
        while tokens.next_if_punctuator(PunctuatorType::CloseBrace).is_none() {
            selections.push(Selection::from_tokens(tokens)?);
        }
        Ok(Self { selections })
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
