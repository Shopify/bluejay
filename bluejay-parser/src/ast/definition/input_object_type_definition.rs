use crate::ast::definition::InputFieldsDefinition;
use crate::ast::{ConstDirectives, FromTokens, ParseError, Tokens, TryFromTokens};
use crate::lexical_token::{Name, StringValue};
use bluejay_core::definition::InputObjectTypeDefinition as CoreInputObjectTypeDefinition;

#[derive(Debug)]
pub struct InputObjectTypeDefinition<'a> {
    description: Option<StringValue>,
    name: Name<'a>,
    directives: Option<ConstDirectives<'a>>,
    input_fields_definition: InputFieldsDefinition<'a>,
}

impl<'a> CoreInputObjectTypeDefinition for InputObjectTypeDefinition<'a> {
    type InputFieldsDefinition = InputFieldsDefinition<'a>;
    type Directives = ConstDirectives<'a>;

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }

    fn input_field_definitions(&self) -> &Self::InputFieldsDefinition {
        &self.input_fields_definition
    }
}

impl<'a> InputObjectTypeDefinition<'a> {
    pub(crate) const INPUT_IDENTIFIER: &'static str = "input";

    pub(crate) fn name(&self) -> &Name<'a> {
        &self.name
    }
}

impl<'a> FromTokens<'a> for InputObjectTypeDefinition<'a> {
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

impl<'a> AsRef<InputObjectTypeDefinition<'a>> for InputObjectTypeDefinition<'a> {
    fn as_ref(&self) -> &InputObjectTypeDefinition<'a> {
        self
    }
}
