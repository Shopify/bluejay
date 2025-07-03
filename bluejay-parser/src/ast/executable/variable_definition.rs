use crate::ast::try_from_tokens::TryFromTokens;
use crate::ast::DepthLimiter;
use crate::ast::{
    executable::VariableType, ConstDirectives, ConstValue, FromTokens, ParseError, Tokens,
};
use crate::lexical_token::{PunctuatorType, StringValue, Variable};

#[derive(Debug)]
pub struct VariableDefinition<'a> {
    description: Option<StringValue<'a>>,
    variable: Variable<'a>,
    r#type: VariableType<'a>,
    default_value: Option<ConstValue<'a>>,
    directives: Option<ConstDirectives<'a>>,
}

impl<'a> FromTokens<'a> for VariableDefinition<'a> {
    #[inline]
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();
        let variable = tokens.expect_variable()?;
        tokens.expect_punctuator(PunctuatorType::Colon)?;
        let r#type = VariableType::from_tokens(tokens, depth_limiter.bump()?)?;
        let default_value: Option<ConstValue> =
            if tokens.next_if_punctuator(PunctuatorType::Equals).is_some() {
                Some(ConstValue::from_tokens(tokens, depth_limiter.bump()?)?)
            } else {
                None
            };
        let directives =
            ConstDirectives::try_from_tokens(tokens, depth_limiter.bump()?).transpose()?;
        Ok(Self {
            description,
            variable,
            r#type,
            default_value,
            directives,
        })
    }
}

impl VariableDefinition<'_> {
    pub fn variable(&self) -> &Variable {
        &self.variable
    }

    pub fn r#type(&self) -> &VariableType {
        &self.r#type
    }

    pub fn default_value(&self) -> Option<&ConstValue> {
        self.default_value.as_ref()
    }
}

impl<'a> bluejay_core::executable::VariableDefinition for VariableDefinition<'a> {
    type Value = ConstValue<'a>;
    type VariableType = VariableType<'a>;
    type Directives = ConstDirectives<'a>;

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn variable(&self) -> &str {
        self.variable.as_str()
    }

    fn r#type(&self) -> &Self::VariableType {
        &self.r#type
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }

    fn default_value(&self) -> Option<&Self::Value> {
        self.default_value.as_ref()
    }
}
