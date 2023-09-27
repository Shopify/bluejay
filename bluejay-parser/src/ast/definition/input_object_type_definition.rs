use crate::ast::definition::{Context, InputFieldsDefinition};
use crate::ast::{ConstDirectives, FromTokens, ParseError, Tokens, TryFromTokens};
use crate::lexical_token::{Name, StringValue};
use bluejay_core::definition::{
    HasDirectives, InputObjectTypeDefinition as CoreInputObjectTypeDefinition,
};

#[derive(Debug)]
pub struct InputObjectTypeDefinition<'a, C: Context> {
    description: Option<StringValue<'a>>,
    name: Name<'a>,
    directives: Option<ConstDirectives<'a>>,
    input_fields_definition: InputFieldsDefinition<'a, C>,
}

impl<'a, C: Context> CoreInputObjectTypeDefinition for InputObjectTypeDefinition<'a, C> {
    type InputFieldsDefinition = InputFieldsDefinition<'a, C>;

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn input_field_definitions(&self) -> &Self::InputFieldsDefinition {
        &self.input_fields_definition
    }
}

impl<'a, C: Context> InputObjectTypeDefinition<'a, C> {
    pub(crate) const INPUT_IDENTIFIER: &'static str = "input";

    pub fn name_token(&self) -> &Name<'a> {
        &self.name
    }
}

impl<'a, C: Context> FromTokens<'a> for InputObjectTypeDefinition<'a, C> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();
        tokens.expect_name_value(Self::INPUT_IDENTIFIER)?;
        let name = tokens.expect_name()?;
        let directives = ConstDirectives::try_from_tokens(tokens).transpose()?;
        let input_fields_definition = InputFieldsDefinition::from_tokens(tokens)?;
        Ok(Self {
            description,
            name,
            directives,
            input_fields_definition,
        })
    }
}

impl<'a, C: Context> HasDirectives for InputObjectTypeDefinition<'a, C> {
    type Directives = ConstDirectives<'a>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}
