use crate::ast::{ConstDirectives, FromTokens, ParseError, Tokens, TryFromTokens};
use crate::lexical_token::{Name, StringValue};
use crate::Span;
use bluejay_core::definition::ScalarTypeDefinition as CoreScalarTypeDefinition;

#[derive(Debug)]
pub struct CustomScalarTypeDefinition<'a> {
    description: Option<StringValue>,
    _scalar_identifier_span: Span,
    name: Name<'a>,
    directives: Option<ConstDirectives<'a>>,
}

impl<'a> CoreScalarTypeDefinition for CustomScalarTypeDefinition<'a> {
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
}

impl<'a> CustomScalarTypeDefinition<'a> {
    pub(crate) const SCALAR_IDENTIFIER: &'static str = "scalar";
}

impl<'a> FromTokens<'a> for CustomScalarTypeDefinition<'a> {
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
        })
    }
}

impl<'a> AsRef<CustomScalarTypeDefinition<'a>> for CustomScalarTypeDefinition<'a> {
    fn as_ref(&self) -> &CustomScalarTypeDefinition<'a> {
        self
    }
}
