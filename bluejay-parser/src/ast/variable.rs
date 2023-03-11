use crate::ast::{FromTokens, IsMatch, ParseError, Tokens};
use crate::lexical_token::{Name, PunctuatorType};
use crate::{HasSpan, Span};

#[derive(Debug)]
pub struct Variable<'a> {
    pub(crate) name: Name<'a>,
    span: Span,
}

impl<'a> IsMatch<'a> for Variable<'a> {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        tokens.peek_punctuator_matches(0, PunctuatorType::Dollar)
    }
}

impl<'a> FromTokens<'a> for Variable<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let dollar_span = tokens.expect_punctuator(PunctuatorType::Dollar)?;
        let name = tokens.expect_name()?;
        let span = dollar_span.merge(&name.span());
        Ok(Self { name, span })
    }
}

impl<'a> HasSpan for Variable<'a> {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl<'a> Variable<'a> {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl<'a> bluejay_core::Variable for Variable<'a> {
    fn name(&self) -> &str {
        self.name.as_ref()
    }
}
