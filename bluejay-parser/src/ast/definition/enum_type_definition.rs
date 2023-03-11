use crate::ast::definition::EnumValueDefinitions;
use crate::ast::{ConstDirectives, FromTokens, ParseError, Tokens, TryFromTokens};
use crate::lexical_token::{Name, StringValue};
use bluejay_core::definition::EnumTypeDefinition as CoreEnumTypeDefinition;

#[derive(Debug)]
pub struct EnumTypeDefinition<'a> {
    description: Option<StringValue>,
    name: Name<'a>,
    directives: Option<ConstDirectives<'a>>,
    enum_value_definitions: EnumValueDefinitions<'a>,
}

impl<'a> CoreEnumTypeDefinition for EnumTypeDefinition<'a> {
    type EnumValueDefinitions = EnumValueDefinitions<'a>;
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

    fn enum_value_definitions(&self) -> &Self::EnumValueDefinitions {
        &self.enum_value_definitions
    }
}

impl<'a> EnumTypeDefinition<'a> {
    pub(crate) const ENUM_IDENTIFIER: &'static str = "enum";

    pub(crate) fn name(&self) -> &Name<'a> {
        &self.name
    }
}

impl<'a> FromTokens<'a> for EnumTypeDefinition<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();
        tokens.expect_name_value(Self::ENUM_IDENTIFIER)?;
        let name = tokens.expect_name()?;
        let directives = ConstDirectives::try_from_tokens(tokens).transpose()?;
        let enum_value_definitions = EnumValueDefinitions::from_tokens(tokens)?;
        Ok(Self {
            description,
            name,
            directives,
            enum_value_definitions,
        })
    }
}

impl<'a> AsRef<EnumTypeDefinition<'a>> for EnumTypeDefinition<'a> {
    fn as_ref(&self) -> &EnumTypeDefinition<'a> {
        self
    }
}
