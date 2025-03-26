use crate::ast::definition::{ArgumentsDefinition, Context, Directives, OutputType};
use crate::ast::{
    ConstDirectives, DepthLimiter, FromTokens, Parse, ParseError, Tokens, TryFromTokens,
};
use crate::lexical_token::{Name, PunctuatorType, StringValue};
use bluejay_core::definition::{FieldDefinition as CoreFieldDefinition, HasDirectives};

#[derive(Debug)]
pub struct FieldDefinition<'a, C: Context> {
    description: Option<StringValue<'a>>,
    name: Name<'a>,
    arguments_definition: Option<ArgumentsDefinition<'a, C>>,
    r#type: OutputType<'a, C>,
    directives: Option<Directives<'a, C>>,
    is_builtin: bool,
}

impl<C: Context> FieldDefinition<'_, C> {
    const __TYPENAME_DEFINITION: &'static str = "__typename: String!";
    const __SCHEMA_DEFINITION: &'static str = "__schema: __Schema!";
    const __TYPE_DEFINITION: &'static str = "__type(name: String!): __Type";

    fn builtin(s: &'static str) -> Self {
        let mut definition = Self::parse(s).unwrap();
        definition.is_builtin = true;
        definition
    }

    pub(crate) fn __typename() -> Self {
        Self::builtin(Self::__TYPENAME_DEFINITION)
    }

    pub(crate) fn __schema() -> Self {
        Self::builtin(Self::__SCHEMA_DEFINITION)
    }

    pub(crate) fn __type() -> Self {
        Self::builtin(Self::__TYPE_DEFINITION)
    }
}

impl<'a, C: Context> CoreFieldDefinition for FieldDefinition<'a, C> {
    type ArgumentsDefinition = ArgumentsDefinition<'a, C>;
    type OutputType = OutputType<'a, C>;

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn arguments_definition(&self) -> Option<&Self::ArgumentsDefinition> {
        self.arguments_definition.as_ref()
    }

    fn r#type(&self) -> &Self::OutputType {
        &self.r#type
    }

    fn is_builtin(&self) -> bool {
        self.is_builtin
    }
}

impl<'a, C: Context> FromTokens<'a> for FieldDefinition<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();
        let name = tokens.expect_name()?;
        let arguments_definition =
            ArgumentsDefinition::try_from_tokens(tokens, depth_limiter.bump()?).transpose()?;
        tokens.expect_punctuator(PunctuatorType::Colon)?;
        let r#type = OutputType::from_tokens(tokens, depth_limiter.bump()?)?;
        let directives =
            ConstDirectives::try_from_tokens(tokens, depth_limiter.bump()?).transpose()?;
        Ok(Self {
            description,
            name,
            arguments_definition,
            r#type,
            directives: directives.map(Directives::from),
            is_builtin: false,
        })
    }
}

impl<'a, C: Context> HasDirectives for FieldDefinition<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}
