use crate::ast::{FromTokens, IsMatch, ParseError, Tokens};
use crate::lexical_token::VariableName;
use crate::{HasSpan, Span};

#[derive(Debug)]
pub struct Variable<'a> {
    name: VariableName<'a>,
    span: Span,
}

impl<'a> IsMatch<'a> for Variable<'a> {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        tokens.peek_variable_name(0)
    }
}

impl<'a> FromTokens<'a> for Variable<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let name = tokens.expect_variable_name()?;
        let span = name.span().to_owned();
        Ok(Self { name, span })
    }
}

impl<'a> HasSpan for Variable<'a> {
    fn span(&self) -> &Span {
        &self.span
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
