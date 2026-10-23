use crate::ast::{
    definition::{Context, Directives, SchemaDefinition},
    ConstDirectives, DepthLimiter, FromTokens, ParseError, Tokens, TryFromTokens,
};
use crate::lexical_token::{Name, StringValue};
use crate::{HasSpan, Span};
use bluejay_core::definition::{HasDirectives, ScalarTypeDefinition as CoreScalarTypeDefinition};
use bluejay_core::Value;
use std::borrow::Cow;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct CustomScalarTypeDefinition<'a, C: Context> {
    description: Option<StringValue<'a>>,
    name: Name<'a>,
    directives: Option<Directives<'a, C>>,
    span: Span,
    context: PhantomData<C>,
}

impl<'a, C: Context> CustomScalarTypeDefinition<'a, C> {
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

impl<'a, C: Context> CoreScalarTypeDefinition for CustomScalarTypeDefinition<'a, C> {
    type SchemaDefinition = SchemaDefinition<'a, C>;

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn coerce_input<const CONST: bool>(
        &self,
        value: &impl Value<CONST>,
    ) -> Result<(), Cow<'static, str>> {
        C::coerce_custom_scalar_input(self, value)
    }
}

impl<C: Context> CustomScalarTypeDefinition<'_, C> {
    pub(crate) const SCALAR_IDENTIFIER: &'static str = "scalar";
}

impl<'a, C: Context> FromTokens<'a> for CustomScalarTypeDefinition<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();
        let scalar_identifier_span = tokens.expect_name_value(Self::SCALAR_IDENTIFIER)?;
        let name = tokens.expect_name()?;
        let directives = ConstDirectives::try_from_tokens(tokens, depth_limiter.bump()?)?;
        let start = description
            .as_ref()
            .map_or(scalar_identifier_span, |description| *description.span());
        let end = directives
            .as_ref()
            .and_then(|directives| directives.span().copied())
            .unwrap_or(*name.span());
        Ok(Self {
            description,
            name,
            directives: directives.map(Directives::from),
            span: start.merge(&end),
            context: Default::default(),
        })
    }
}

impl<C: Context> HasSpan for CustomScalarTypeDefinition<'_, C> {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl<'a, C: Context> HasDirectives for CustomScalarTypeDefinition<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}
