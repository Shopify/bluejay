use crate::ast::{FromTokens, ParseError, Tokens, Value};
use crate::lexical_token::{Name, PunctuatorType};
use crate::{HasSpan, Span};

#[derive(Debug)]
pub struct Argument<'a, const CONST: bool> {
    name: Name<'a>,
    value: Value<'a, CONST>,
    span: Span,
}

impl<'a, const CONST: bool> Argument<'a, CONST> {
    pub fn name(&self) -> &Name<'a> {
        &self.name
    }

    pub fn value(&self) -> &Value<'a, CONST> {
        &self.value
    }
}

pub type ConstArgument<'a> = Argument<'a, true>;
pub type VariableArgument<'a> = Argument<'a, false>;

impl<'a, const CONST: bool> FromTokens<'a> for Argument<'a, CONST> {
    #[inline]
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let name = tokens.expect_name()?;
        tokens.expect_punctuator(PunctuatorType::Colon)?;
        let value = Value::from_tokens(tokens)?;
        let span = name.span().merge(value.span());
        Ok(Self { name, value, span })
    }
}

impl<'a, const CONST: bool> bluejay_core::Argument<CONST> for Argument<'a, CONST> {
    type Value = Value<'a, CONST>;

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn value(&self) -> &Value<'a, CONST> {
        &self.value
    }
}

impl<'a, const CONST: bool> HasSpan for Argument<'a, CONST> {
    fn span(&self) -> &Span {
        &self.span
    }
}
