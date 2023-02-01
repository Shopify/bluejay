use crate::ast::{Argument, FromTokens, IsMatch, ParseError, Tokens};
use crate::lexical_token::{HasSpan, PunctuatorType};
use crate::Span;

#[derive(Debug)]
pub struct Arguments<'a, const CONST: bool> {
    arguments: Vec<Argument<'a, CONST>>,
    span: Span,
}

pub type VariableArguments<'a> = Arguments<'a, false>;

impl<'a, const CONST: bool> FromTokens<'a> for Arguments<'a, CONST> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let open_span = tokens.expect_punctuator(PunctuatorType::OpenRoundBracket)?;
        let mut arguments: Vec<Argument<CONST>> = Vec::new();
        let close_span = loop {
            if let Some(close_span) = tokens.next_if_punctuator(PunctuatorType::CloseRoundBracket) {
                break close_span;
            }
            arguments.push(Argument::from_tokens(tokens)?);
        };
        let span = open_span.merge(&close_span);
        Ok(Self { arguments, span })
    }
}

impl<'a, const CONST: bool> IsMatch<'a> for Arguments<'a, CONST> {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        tokens.peek_punctuator_matches(0, PunctuatorType::OpenRoundBracket)
    }
}

impl<'a, const CONST: bool> HasSpan for Arguments<'a, CONST> {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl<'a, const CONST: bool> bluejay_core::Arguments<CONST> for Arguments<'a, CONST> {
    type Argument = Argument<'a, CONST>;
}

impl<'a, const CONST: bool> AsRef<[Argument<'a, CONST>]> for Arguments<'a, CONST> {
    fn as_ref(&self) -> &[Argument<'a, CONST>] {
        &self.arguments
    }
}
