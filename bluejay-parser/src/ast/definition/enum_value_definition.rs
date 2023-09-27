use crate::ast::{ConstDirectives, FromTokens, ParseError, Tokens, TryFromTokens};
use crate::lexical_token::{Name, StringValue};
use bluejay_core::definition::{EnumValueDefinition as CoreEnumValueDefinition, HasDirectives};

#[derive(Debug)]
pub struct EnumValueDefinition<'a> {
    description: Option<StringValue<'a>>,
    name: Name<'a>,
    directives: Option<ConstDirectives<'a>>,
}

impl<'a> EnumValueDefinition<'a> {
    pub fn name_token(&self) -> &Name<'a> {
        &self.name
    }
}

impl<'a> CoreEnumValueDefinition for EnumValueDefinition<'a> {
    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl<'a> FromTokens<'a> for EnumValueDefinition<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();
        let name = tokens.expect_name()?;
        let directives = ConstDirectives::try_from_tokens(tokens).transpose()?;
        Ok(Self {
            description,
            name,
            directives,
        })
    }
}

impl<'a> HasDirectives for EnumValueDefinition<'a> {
    type Directives = ConstDirectives<'a>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}
