use crate::ast::definition::{FieldsDefinition, InterfaceImplementations};
use crate::ast::{ConstDirectives, FromTokens, ParseError, Tokens, TryFromTokens};
use crate::lexical_token::{Name, StringValue};
use bluejay_core::definition::ObjectTypeDefinition as CoreObjectTypeDefinition;

#[derive(Debug)]
pub struct ObjectTypeDefinition<'a> {
    description: Option<StringValue>,
    name: Name<'a>,
    interface_implementations: Option<InterfaceImplementations<'a>>,
    directives: Option<ConstDirectives<'a>>,
    fields_definition: FieldsDefinition<'a>,
}

impl<'a> CoreObjectTypeDefinition for ObjectTypeDefinition<'a> {
    type FieldsDefinition = FieldsDefinition<'a>;
    type InterfaceImplementations = InterfaceImplementations<'a>;
    type Directives = ConstDirectives<'a>;

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn interface_implementations(&self) -> Option<&Self::InterfaceImplementations> {
        self.interface_implementations.as_ref()
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }

    fn fields_definition(&self) -> &Self::FieldsDefinition {
        &self.fields_definition
    }
}

impl<'a> ObjectTypeDefinition<'a> {
    pub(crate) const TYPE_IDENTIFIER: &'static str = "type";
}

impl<'a> FromTokens<'a> for ObjectTypeDefinition<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();
        tokens.expect_name_value(Self::TYPE_IDENTIFIER)?;
        let name = tokens.expect_name()?;
        let interface_implementations =
            InterfaceImplementations::try_from_tokens(tokens).transpose()?;
        let directives = ConstDirectives::try_from_tokens(tokens).transpose()?;
        let fields_definition = FieldsDefinition::from_tokens(tokens)?;
        Ok(Self {
            description,
            name,
            interface_implementations,
            directives,
            fields_definition,
        })
    }
}

impl<'a> AsRef<ObjectTypeDefinition<'a>> for ObjectTypeDefinition<'a> {
    fn as_ref(&self) -> &ObjectTypeDefinition<'a> {
        self
    }
}