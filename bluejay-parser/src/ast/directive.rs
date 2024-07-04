use crate::ast::{Arguments, FromTokens, IsMatch, ParseError, Tokens, TryFromTokens};
use crate::lexical_token::Name;
use crate::{HasSpan, Span};

#[derive(Debug)]
pub struct Directive<'a, const CONST: bool> {
    name: Name<'a>,
    arguments: Option<Arguments<'a, CONST>>,
    span: Span,
}

pub type ConstDirective<'a> = Directive<'a, true>;
pub type VariableDirective<'a> = Directive<'a, false>;

impl<'a, const CONST: bool> IsMatch<'a> for Directive<'a, CONST> {
    #[inline]
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        tokens.peek_directive_name(0)
    }
}

impl<'a, const CONST: bool> FromTokens<'a> for Directive<'a, CONST> {
    #[inline]
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let name = tokens.expect_directive_name()?;
        let arguments = Arguments::try_from_tokens(tokens).transpose()?;
        let span = match &arguments {
            Some(arguments) => name.span().merge(arguments.span()),
            None => name.span().to_owned(),
        };

        Ok(Self {
            name,
            arguments,
            span,
        })
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
    fn span(&self) -> &Span {
        &self.span
    }
}

impl<'a, const CONST: bool> Directive<'a, CONST> {
    pub fn name(&self) -> &Name<'a> {
        &self.name
    }
}
