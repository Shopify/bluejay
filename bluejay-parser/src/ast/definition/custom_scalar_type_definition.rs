use crate::ast::{
    definition::Context, ConstDirectives, FromTokens, ParseError, Tokens, TryFromTokens,
};
use crate::lexical_token::{Name, StringValue};
use crate::Span;
use bluejay_core::definition::ScalarTypeDefinition as CoreScalarTypeDefinition;
use bluejay_core::AbstractValue;
use std::borrow::Cow;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct CustomScalarTypeDefinition<'a, C: Context> {
    description: Option<StringValue<'a>>,
    _scalar_identifier_span: Span,
    name: Name<'a>,
    directives: Option<ConstDirectives<'a>>,
    context: PhantomData<C>,
}

impl<'a, C: Context> CustomScalarTypeDefinition<'a, C> {
    pub(crate) fn name(&self) -> &Name<'a> {
        &self.name
    }
}

impl<'a, C: Context> CoreScalarTypeDefinition for CustomScalarTypeDefinition<'a, C> {
    type Directives = ConstDirectives<'a>;

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }

    fn coerce_input<const CONST: bool>(
        &self,
        value: &impl AbstractValue<CONST>,
    ) -> Result<(), Cow<'static, str>> {
        C::coerce_custom_scalar_input(self, value)
    }
}

impl<'a, C: Context> CustomScalarTypeDefinition<'a, C> {
    pub(crate) const SCALAR_IDENTIFIER: &'static str = "scalar";
}

impl<'a, C: Context> FromTokens<'a> for CustomScalarTypeDefinition<'a, C> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();
        let scalar_identifier_span = tokens.expect_name_value(Self::SCALAR_IDENTIFIER)?;
        let name = tokens.expect_name()?;
        let directives = ConstDirectives::try_from_tokens(tokens).transpose()?;
        Ok(Self {
            description,
            _scalar_identifier_span: scalar_identifier_span,
            name,
            directives,
            context: Default::default(),
        })
    }
}
