use crate::ast::definition::{Context, Directives, FieldsDefinition, InterfaceImplementations};
use crate::ast::{ConstDirectives, DepthLimiter, FromTokens, ParseError, Tokens, TryFromTokens};
use crate::lexical_token::{Name, StringValue};
use bluejay_core::definition::{
    HasDirectives, InterfaceTypeDefinition as CoreInterfaceTypeDefinition,
};

#[derive(Debug)]
pub struct InterfaceTypeDefinition<'a, C: Context> {
    description: Option<StringValue<'a>>,
    name: Name<'a>,
    interface_implementations: Option<InterfaceImplementations<'a, C>>,
    directives: Option<Directives<'a, C>>,
    fields_definition: FieldsDefinition<'a, C>,
}

impl<'a, C: Context> CoreInterfaceTypeDefinition for InterfaceTypeDefinition<'a, C> {
    type FieldsDefinition = FieldsDefinition<'a, C>;
    type InterfaceImplementations = InterfaceImplementations<'a, C>;

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn interface_implementations(&self) -> Option<&Self::InterfaceImplementations> {
        self.interface_implementations.as_ref()
    }

    fn fields_definition(&self) -> &Self::FieldsDefinition {
        &self.fields_definition
    }
}

impl<'a, C: Context> InterfaceTypeDefinition<'a, C> {
    pub(crate) const INTERFACE_IDENTIFIER: &'static str = "interface";

    pub(crate) fn name(&self) -> &Name<'a> {
        &self.name
    }
}

impl<'a, C: Context> FromTokens<'a> for InterfaceTypeDefinition<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();
        tokens.expect_name_value(Self::INTERFACE_IDENTIFIER)?;
        let name = tokens.expect_name()?;
        let interface_implementations =
            InterfaceImplementations::try_from_tokens(tokens, depth_limiter.bump()?).transpose()?;
        let directives =
            ConstDirectives::try_from_tokens(tokens, depth_limiter.bump()?).transpose()?;
        let fields_definition = FieldsDefinition::from_tokens(tokens, depth_limiter.bump()?)?;
        Ok(Self {
            description,
            name,
            interface_implementations,
            directives: directives.map(Directives::from),
            fields_definition,
        })
    }
}

impl<'a, C: Context> HasDirectives for InterfaceTypeDefinition<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}
