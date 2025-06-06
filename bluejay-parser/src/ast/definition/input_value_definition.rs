use crate::ast::definition::{Context, Directives, InputType};
use crate::ast::{ConstDirectives, ConstValue, DepthLimiter, FromTokens, ParseError, Tokens};
use crate::lexical_token::{Name, PunctuatorType, StringValue};
use bluejay_core::definition::{HasDirectives, InputValueDefinition as CoreInputValueDefinition};

#[derive(Debug)]
pub struct InputValueDefinition<'a, C: Context> {
    description: Option<StringValue<'a>>,
    name: Name<'a>,
    r#type: InputType<'a, C>,
    default_value: Option<ConstValue<'a>>,
    directives: Option<Directives<'a, C>>,
}

impl<'a, C: Context> InputValueDefinition<'a, C> {
    pub fn name_token(&self) -> &Name<'a> {
        &self.name
    }
}

impl<'a, C: Context> CoreInputValueDefinition for InputValueDefinition<'a, C> {
    type InputType = InputType<'a, C>;
    type Value = ConstValue<'a>;

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn r#type(&self) -> &Self::InputType {
        &self.r#type
    }

    fn default_value(&self) -> Option<&Self::Value> {
        self.default_value.as_ref()
    }
}

impl<'a, C: Context> FromTokens<'a> for InputValueDefinition<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();
        let name = tokens.expect_name()?;
        tokens.expect_punctuator(PunctuatorType::Colon)?;
        let r#type = InputType::from_tokens(tokens, depth_limiter.bump()?)?;
        let default_value: Option<ConstValue> =
            if tokens.next_if_punctuator(PunctuatorType::Equals).is_some() {
                Some(ConstValue::from_tokens(tokens, depth_limiter.bump()?)?)
            } else {
                None
            };
        let directives = Some(ConstDirectives::from_tokens(tokens, depth_limiter.bump()?)?);
        Ok(Self {
            description,
            name,
            r#type,
            default_value,
            directives: directives.map(Directives::from),
        })
    }
}

impl<'a, C: Context> HasDirectives for InputValueDefinition<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}
