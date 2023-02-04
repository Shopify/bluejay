use crate::ast::{FromTokens, ParseError, Tokens, Value};
use crate::lexical_token::{Name, PunctuatorType};

#[derive(Debug)]
pub struct Argument<'a, const CONST: bool> {
    pub(crate) name: Name<'a>,
    pub(crate) value: Value<'a, CONST>,
}

pub type ConstArgument<'a> = Argument<'a, true>;
pub type VariableArgument<'a> = Argument<'a, false>;

impl<'a, const CONST: bool> FromTokens<'a> for Argument<'a, CONST> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let name = tokens.expect_name()?;
        tokens.expect_punctuator(PunctuatorType::Colon)?;
        let value = Value::from_tokens(tokens)?;
        Ok(Self { name, value })
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
