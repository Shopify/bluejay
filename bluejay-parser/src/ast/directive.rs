use crate::ast::{Arguments, FromTokens, IsMatch, ParseError, Tokens, TryFromTokens};
use crate::lexical_token::{Name, PunctuatorType};
use crate::{HasSpan, Span};

#[derive(Debug)]
pub struct Directive<'a, const CONST: bool> {
    name: Name<'a>,
    arguments: Option<Arguments<'a, CONST>>,
}

impl<'a, const CONST: bool> IsMatch<'a> for Directive<'a, CONST> {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        tokens.peek_punctuator_matches(0, PunctuatorType::At)
    }
}

impl<'a, const CONST: bool> FromTokens<'a> for Directive<'a, CONST> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        tokens.expect_punctuator(PunctuatorType::At)?;
        let name = tokens.expect_name()?;
        let arguments = Arguments::try_from_tokens(tokens).transpose()?;

        Ok(Self { name, arguments })
    }
}

impl<'a, const CONST: bool> bluejay_core::Directive<CONST> for Directive<'a, CONST> {
    type Arguments = Arguments<'a, CONST>;

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn arguments(&self) -> Option<&Self::Arguments> {
        self.arguments.as_ref()
    }
}

impl<'a, const CONST: bool> HasSpan for Directive<'a, CONST> {
    fn span(&self) -> Span {
        match &self.arguments {
            Some(arguments) => self.name.span().merge(&arguments.span()),
            None => self.name.span(),
        }
    }
}
