use crate::ast::definition::{ArgumentsDefinition, Context, OutputTypeReference};
use crate::ast::{ConstDirectives, FromTokens, ParseError, Tokens, TryFromTokens};
use crate::lexical_token::{Name, PunctuatorType, StringValue};
use crate::Span;
use bluejay_core::definition::FieldDefinition as CoreFieldDefinition;

#[derive(Debug)]
pub struct FieldDefinition<'a, C: Context> {
    description: Option<StringValue<'a>>,
    name: Name<'a>,
    arguments_definition: Option<ArgumentsDefinition<'a, C>>,
    r#type: OutputTypeReference<'a, C>,
    directives: Option<ConstDirectives<'a>>,
    is_builtin: bool,
}

impl<'a, C: Context> FieldDefinition<'a, C> {
    pub(crate) fn typename() -> Self {
        FieldDefinition {
            description: None,
            name: Name::new("__typename", Span::empty()),
            arguments_definition: None,
            r#type: OutputTypeReference::non_null_string(),
            directives: None,
            is_builtin: true,
        }
    }
}

impl<'a, C: Context> CoreFieldDefinition for FieldDefinition<'a, C> {
    type ArgumentsDefinition = ArgumentsDefinition<'a, C>;
    type OutputTypeReference = OutputTypeReference<'a, C>;
    type Directives = ConstDirectives<'a>;

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn arguments_definition(&self) -> Option<&Self::ArgumentsDefinition> {
        self.arguments_definition.as_ref()
    }

    fn r#type(&self) -> &Self::OutputTypeReference {
        &self.r#type
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }

    fn is_builtin(&self) -> bool {
        self.is_builtin
    }
}

impl<'a, C: Context> FromTokens<'a> for FieldDefinition<'a, C> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();
        let name = tokens.expect_name()?;
        let arguments_definition = ArgumentsDefinition::try_from_tokens(tokens).transpose()?;
        tokens.expect_punctuator(PunctuatorType::Colon)?;
        let r#type = OutputTypeReference::from_tokens(tokens)?;
        let directives = ConstDirectives::try_from_tokens(tokens).transpose()?;
        Ok(Self {
            description,
            name,
            arguments_definition,
            r#type,
            directives,
            is_builtin: false,
        })
    }
}
