use crate::ast::executable::{Field, FragmentSpread, InlineFragment};
use crate::ast::{DepthLimiter, FromTokens, IsMatch, ParseError, Tokens};
use crate::lexical_token::PunctuatorType;
use bluejay_core::executable::{Selection as CoreSelection, SelectionReference};

#[derive(Debug)]
pub enum Selection<'a> {
    Field(Field<'a>),
    FragmentSpread(FragmentSpread<'a>),
    InlineFragment(InlineFragment<'a>),
}

impl<'a> CoreSelection for Selection<'a> {
    type Field = Field<'a>;
    type FragmentSpread = FragmentSpread<'a>;
    type InlineFragment = InlineFragment<'a>;

    fn as_ref(&self) -> SelectionReference<'_, Self> {
        match self {
            Self::Field(f) => SelectionReference::Field(f),
            Self::FragmentSpread(fs) => SelectionReference::FragmentSpread(fs),
            Self::InlineFragment(i) => SelectionReference::InlineFragment(i),
        }
    }
}

impl<'a> FromTokens<'a> for Selection<'a> {
    #[inline]
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        // don't bump depth limiters because this is just a thin wrapper in the AST
        if Field::is_match(tokens) {
            Field::from_tokens(tokens, depth_limiter).map(Self::Field)
        } else if FragmentSpread::is_match(tokens) {
            FragmentSpread::from_tokens(tokens, depth_limiter).map(Self::FragmentSpread)
        } else if InlineFragment::is_match(tokens) {
            InlineFragment::from_tokens(tokens, depth_limiter).map(Self::InlineFragment)
        } else {
            Err(tokens.unexpected_token())
        }
    }
}

impl<'a> IsMatch<'a> for Selection<'a> {
    #[inline]
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        Field::is_match(tokens) || tokens.peek_punctuator_matches(0, PunctuatorType::Ellipse)
    }
}
