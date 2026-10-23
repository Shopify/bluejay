use crate::ast::definition::{
    Context, Directives, FieldsDefinition, InterfaceImplementations, SchemaDefinition,
};
use crate::ast::{ConstDirectives, DepthLimiter, FromTokens, ParseError, Tokens, TryFromTokens};
use crate::lexical_token::{Name, PunctuatorType, StringValue};
use crate::{HasSpan, Span};
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
    span: Span,
}

impl<'a, C: Context> CoreInterfaceTypeDefinition for InterfaceTypeDefinition<'a, C> {
    type SchemaDefinition = SchemaDefinition<'a, C>;

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn interface_implementations(&self) -> Option<&InterfaceImplementations<'a, C>> {
        self.interface_implementations.as_ref()
    }

    fn fields_definition(&self) -> &FieldsDefinition<'a, C> {
        &self.fields_definition
    }
}

impl<'a, C: Context> InterfaceTypeDefinition<'a, C> {
    pub(crate) const INTERFACE_IDENTIFIER: &'static str = "interface";

    pub(crate) fn name(&self) -> &Name<'a> {
        &self.name
    }

    pub fn name_token(&self) -> &Name<'a> {
        &self.name
    }

    pub fn description_token(&self) -> Option<&StringValue<'a>> {
        self.description.as_ref()
    }
}

impl<'a, C: Context> FromTokens<'a> for InterfaceTypeDefinition<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();
        let interface_identifier_span = tokens.expect_name_value(Self::INTERFACE_IDENTIFIER)?;
        let name = tokens.expect_name()?;
        let interface_implementations =
            InterfaceImplementations::try_from_tokens(tokens, depth_limiter.bump()?)?;
        let directives = ConstDirectives::try_from_tokens(tokens, depth_limiter.bump()?)?;
        // Optional in the grammar. `SchemaDefinition::try_from` rejects an empty type.
        let fields_definition = if tokens.peek_punctuator_matches(0, PunctuatorType::OpenBrace) {
            Some(FieldsDefinition::from_tokens(
                tokens,
                depth_limiter.bump()?,
            )?)
        } else {
            None
        };
        let start = description
            .as_ref()
            .map_or(interface_identifier_span, |description| *description.span());
        let end = fields_definition
            .as_ref()
            .map(|fields_definition| *fields_definition.span())
            .or_else(|| {
                directives
                    .as_ref()
                    .and_then(|directives| directives.span().copied())
            })
            .or_else(|| {
                interface_implementations
                    .as_ref()
                    .and_then(InterfaceImplementations::span)
            })
            .unwrap_or(*name.span());
        Ok(Self {
            description,
            name,
            interface_implementations,
            directives: directives.map(Directives::from),
            fields_definition: fields_definition.unwrap_or_else(FieldsDefinition::__typename),
            span: start.merge(&end),
        })
    }
}

impl<C: Context> HasSpan for InterfaceTypeDefinition<'_, C> {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl<'a, C: Context> HasDirectives for InterfaceTypeDefinition<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}
