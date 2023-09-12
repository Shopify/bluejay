use crate::ast::definition::{ArgumentsDefinition, Context};
use crate::ast::{FromTokens, Parse, ParseError, Tokens, TryFromTokens};
use crate::lexical_token::{Name, PunctuatorType, StringValue};
use crate::Span;
use bluejay_core::definition::{
    DirectiveDefinition as CoreDirectiveDefinition, DirectiveLocation as CoreDirectiveLocation,
};
use bluejay_core::AsIter;
use std::str::FromStr;

#[derive(Debug)]
pub struct DirectiveDefinition<'a, C: Context> {
    description: Option<StringValue<'a>>,
    name: Name<'a>,
    arguments_definition: Option<ArgumentsDefinition<'a, C>>,
    is_repeatable: bool,
    locations: DirectiveLocations,
    is_builtin: bool,
}

impl<'a, C: Context> CoreDirectiveDefinition for DirectiveDefinition<'a, C> {
    type ArgumentsDefinition = ArgumentsDefinition<'a, C>;
    type DirectiveLocations = DirectiveLocations;

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn arguments_definition(&self) -> Option<&Self::ArgumentsDefinition> {
        self.arguments_definition.as_ref()
    }

    fn is_repeatable(&self) -> bool {
        self.is_repeatable
    }

    fn locations(&self) -> &Self::DirectiveLocations {
        &self.locations
    }

    fn is_builtin(&self) -> bool {
        self.is_builtin
    }
}

impl<'a, C: Context> DirectiveDefinition<'a, C> {
    pub(crate) const DIRECTIVE_IDENTIFIER: &'static str = "directive";
    const REPEATABLE_IDENTIFIER: &'static str = "repeatable";
    const ON_IDENTIFIER: &'static str = "on";
    const SKIP_DEFINITION: &'static str =
        "directive @skip(if: Boolean!) on FIELD | FRAGMENT_SPREAD | INLINE_FRAGMENT";
    const INCLUDE_DEFINITION: &'static str =
        "directive @include(if: Boolean!) on FIELD | FRAGMENT_SPREAD | INLINE_FRAGMENT";
    const DEPRECATED_DEFINITION: &'static str = "directive @deprecated(reason: String = \"No longer supported\") on FIELD_DEFINITION | ARGUMENT_DEFINITION | INPUT_FIELD_DEFINITION | ENUM_VALUE";
    const SPECIFIED_BY_DEFINITION: &'static str = "directive @specifiedBy(url: String!) on SCALAR";

    fn builtin(s: &'static str) -> Self {
        let mut definition = DirectiveDefinition::parse(s).unwrap();
        definition.is_builtin = true;
        definition
    }

    pub(crate) fn skip() -> Self {
        Self::builtin(Self::SKIP_DEFINITION)
    }

    pub(crate) fn include() -> Self {
        Self::builtin(Self::INCLUDE_DEFINITION)
    }

    pub(crate) fn deprecated() -> Self {
        Self::builtin(Self::DEPRECATED_DEFINITION)
    }

    pub(crate) fn specified_by() -> Self {
        Self::builtin(Self::SPECIFIED_BY_DEFINITION)
    }

    pub(crate) fn name_token(&self) -> &Name<'a> {
        &self.name
    }
}

impl<'a, C: Context> FromTokens<'a> for DirectiveDefinition<'a, C> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();
        tokens.expect_name_value(Self::DIRECTIVE_IDENTIFIER)?;
        tokens.expect_punctuator(PunctuatorType::At)?;
        let name = tokens.expect_name()?;
        let arguments_definition = ArgumentsDefinition::try_from_tokens(tokens).transpose()?;
        let is_repeatable = tokens
            .next_if_name_matches(Self::REPEATABLE_IDENTIFIER)
            .is_some();
        tokens.expect_name_value(Self::ON_IDENTIFIER)?;
        let locations = DirectiveLocations::from_tokens(tokens)?;
        Ok(Self {
            description,
            name,
            arguments_definition,
            is_repeatable,
            locations,
            is_builtin: false,
        })
    }
}

#[derive(Debug)]
pub struct DirectiveLocation {
    inner: CoreDirectiveLocation,
    _span: Span,
}

impl<'a> FromTokens<'a> for DirectiveLocation {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        tokens.expect_name().and_then(
            |name| match CoreDirectiveLocation::from_str(name.as_ref()) {
                Ok(inner) => Ok(Self {
                    inner,
                    _span: name.into(),
                }),
                Err(_) => Err(ParseError::ExpectedOneOf {
                    span: name.into(),
                    values: CoreDirectiveLocation::POSSIBLE_VALUES,
                }),
            },
        )
    }
}

impl AsRef<CoreDirectiveLocation> for DirectiveLocation {
    fn as_ref(&self) -> &CoreDirectiveLocation {
        &self.inner
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct DirectiveLocations(Vec<DirectiveLocation>);

impl AsIter for DirectiveLocations {
    type Item = CoreDirectiveLocation;
    type Iterator<'a> = std::iter::Map<
        std::slice::Iter<'a, DirectiveLocation>,
        fn(&'a DirectiveLocation) -> &'a CoreDirectiveLocation,
    >;

    fn iter(&self) -> Self::Iterator<'_> {
        self.0.iter().map(AsRef::as_ref)
    }
}

impl<'a> FromTokens<'a> for DirectiveLocations {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        tokens.next_if_punctuator(PunctuatorType::Pipe);
        let mut directive_locations: Vec<DirectiveLocation> =
            vec![DirectiveLocation::from_tokens(tokens)?];
        while tokens.next_if_punctuator(PunctuatorType::Pipe).is_some() {
            directive_locations.push(DirectiveLocation::from_tokens(tokens)?);
        }
        Ok(Self(directive_locations))
    }
}
