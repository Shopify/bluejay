use crate::ast::definition::InputTypeReference;
use crate::ast::{ConstDirectives, ConstValue, FromTokens, ParseError, Tokens};
use crate::lexical_token::{Name, PunctuatorType, StringValue};
use bluejay_core::definition::InputValueDefinition as CoreInputValueDefinition;

#[derive(Debug)]
pub struct InputValueDefinition<'a> {
    description: Option<StringValue>,
    name: Name<'a>,
    r#type: InputTypeReference<'a>,
    default_value: Option<ConstValue<'a>>,
    directives: Option<ConstDirectives<'a>>,
}

impl<'a> CoreInputValueDefinition for InputValueDefinition<'a> {
    type InputTypeReference = InputTypeReference<'a>;
    type Value = ConstValue<'a>;
    type Directives = ConstDirectives<'a>;

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn r#type(&self) -> &Self::InputTypeReference {
        &self.r#type
    }

    fn default_value(&self) -> Option<&Self::Value> {
        self.default_value.as_ref()
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}

impl<'a> FromTokens<'a> for InputValueDefinition<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();
        let name = tokens.expect_name()?;
        tokens.expect_punctuator(PunctuatorType::Colon)?;
        let r#type = InputTypeReference::from_tokens(tokens)?;
        let default_value: Option<ConstValue> =
            if tokens.next_if_punctuator(PunctuatorType::Equals).is_some() {
                Some(ConstValue::from_tokens(tokens)?)
            } else {
                None
            };
        let directives = Some(ConstDirectives::from_tokens(tokens)?);
        Ok(Self {
            description,
            name,
            r#type,
            default_value,
            directives,
        })
    }
}
