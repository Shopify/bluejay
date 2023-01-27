use crate::ast::executable::{Field, FragmentSpread, InlineFragment};
use crate::ast::{ParseError, Tokens, IsMatch, FromTokens};
use crate::lexical_token::PunctuatorType;

pub type Selection<'a> = bluejay_core::executable::Selection<Field<'a>, FragmentSpread<'a>, InlineFragment<'a>>;

impl<'a> FromTokens<'a> for Selection<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        if Field::is_match(tokens) {
            Field::from_tokens(tokens).map(Self::Field)
        } else if FragmentSpread::is_match(tokens) {
            FragmentSpread::from_tokens(tokens).map(Self::FragmentSpread)
        } else if InlineFragment::is_match(tokens) {
            InlineFragment::from_tokens(tokens).map(Self::InlineFragment)
        } else {
            Err(tokens.unexpected_token())
        }
    }
}

impl<'a> IsMatch<'a> for Selection<'a> {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        Field::is_match(tokens) || tokens.peek_punctuator_matches(0, PunctuatorType::Ellipse)
    }
}
