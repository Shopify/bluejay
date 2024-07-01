use crate::lexical_token::{Name, StringValue};
use crate::{
    ast::{
        definition::{Context, Directives},
        ConstDirectives, FromTokens, ParseError, Tokens, TryFromTokens,
    },
    HasSpan,
};
use bluejay_core::definition::{EnumValueDefinition as CoreEnumValueDefinition, HasDirectives};

#[derive(Debug)]
pub struct EnumValueDefinition<'a, C: Context> {
    description: Option<StringValue<'a>>,
    name: Name<'a>,
    directives: Option<Directives<'a, C>>,
}

impl<'a, C: Context> EnumValueDefinition<'a, C> {
    pub fn name_token(&self) -> &Name<'a> {
        &self.name
    }
}

impl<'a, C: Context> CoreEnumValueDefinition for EnumValueDefinition<'a, C> {
    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl<'a, C: Context> FromTokens<'a> for EnumValueDefinition<'a, C> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();
        let name = tokens.expect_name()?;
        if matches!(name.as_str(), "null" | "true" | "false") {
            return Err(ParseError::InvalidEnumValue {
                span: name.span().clone(),
                value: name.as_str().to_string(),
            });
        }

        let directives = ConstDirectives::try_from_tokens(tokens).transpose()?;
        Ok(Self {
            description,
            name,
            directives: directives.map(Directives::from),
        })
    }
}

impl<'a, C: Context> HasDirectives for EnumValueDefinition<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}
