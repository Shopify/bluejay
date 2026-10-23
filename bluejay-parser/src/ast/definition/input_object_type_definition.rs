use crate::ast::definition::{Context, Directives, InputFieldsDefinition, SchemaDefinition};
use crate::ast::{ConstDirectives, DepthLimiter, FromTokens, ParseError, Tokens, TryFromTokens};
use crate::lexical_token::{Name, PunctuatorType, StringValue};
use crate::{HasSpan, Span};
use bluejay_core::definition::{
    HasDirectives, InputObjectTypeDefinition as CoreInputObjectTypeDefinition,
};

#[derive(Debug)]
pub struct InputObjectTypeDefinition<'a, C: Context> {
    description: Option<StringValue<'a>>,
    name: Name<'a>,
    directives: Option<Directives<'a, C>>,
    input_fields_definition: InputFieldsDefinition<'a, C>,
    span: Span,
}

impl<'a, C: Context> CoreInputObjectTypeDefinition for InputObjectTypeDefinition<'a, C> {
    type SchemaDefinition = SchemaDefinition<'a, C>;

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn input_field_definitions(&self) -> &InputFieldsDefinition<'a, C> {
        &self.input_fields_definition
    }
}

impl<'a, C: Context> InputObjectTypeDefinition<'a, C> {
    pub(crate) const INPUT_IDENTIFIER: &'static str = "input";

    pub fn name_token(&self) -> &Name<'a> {
        &self.name
    }

    pub fn description_token(&self) -> Option<&StringValue<'a>> {
        self.description.as_ref()
    }
}

impl<'a, C: Context> FromTokens<'a> for InputObjectTypeDefinition<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();
        let input_identifier_span = tokens.expect_name_value(Self::INPUT_IDENTIFIER)?;
        let name = tokens.expect_name()?;
        let directives = ConstDirectives::try_from_tokens(tokens, depth_limiter.bump()?)?;
        // Optional in the grammar. `SchemaDefinition::try_from` rejects an empty type.
        let input_fields_definition =
            if tokens.peek_punctuator_matches(0, PunctuatorType::OpenBrace) {
                Some(InputFieldsDefinition::from_tokens(
                    tokens,
                    depth_limiter.bump()?,
                )?)
            } else {
                None
            };
        let start = description
            .as_ref()
            .map_or(input_identifier_span, |description| *description.span());
        let end = input_fields_definition
            .as_ref()
            .map(|input_fields_definition| *input_fields_definition.span())
            .or_else(|| {
                directives
                    .as_ref()
                    .and_then(|directives| directives.span().copied())
            })
            .unwrap_or(*name.span());
        Ok(Self {
            description,
            name,
            directives: directives.map(Directives::from),
            input_fields_definition: input_fields_definition
                .unwrap_or_else(InputFieldsDefinition::empty),
            span: start.merge(&end),
        })
    }
}

impl<C: Context> HasSpan for InputObjectTypeDefinition<'_, C> {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl<'a, C: Context> HasDirectives for InputObjectTypeDefinition<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}
