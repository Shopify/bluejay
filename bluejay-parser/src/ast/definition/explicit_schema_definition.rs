use crate::ast::{ConstDirectives, FromTokens, ParseError, Tokens, TryFromTokens};
use crate::lexical_token::{HasSpan, Name, PunctuatorType, StringValue};
use crate::Span;
use bluejay_core::OperationType;
use std::str::FromStr;

#[derive(Debug)]
pub struct ExplicitSchemaDefinition<'a> {
    description: Option<StringValue>,
    _schema_span: Span,
    directives: Option<ConstDirectives<'a>>,
    root_operation_type_definitions: Vec<RootOperationTypeDefinition<'a>>,
}

impl<'a> ExplicitSchemaDefinition<'a> {
    pub(crate) const SCHEMA_IDENTIFIER: &'static str = "schema";

    pub(crate) fn description(&self) -> Option<&StringValue> {
        self.description.as_ref()
    }

    pub(crate) fn root_operation_type_definitions(&self) -> &[RootOperationTypeDefinition<'a>] {
        &self.root_operation_type_definitions
    }

    pub(crate) fn directives(&self) -> Option<&ConstDirectives<'a>> {
        self.directives.as_ref()
    }
}

impl<'a> FromTokens<'a> for ExplicitSchemaDefinition<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();

        let schema_span = tokens.expect_name_value(Self::SCHEMA_IDENTIFIER)?;

        let directives = ConstDirectives::try_from_tokens(tokens).transpose()?;

        let mut root_operation_type_definitions = Vec::new();

        tokens.expect_punctuator(PunctuatorType::OpenBrace)?;

        while tokens
            .next_if_punctuator(PunctuatorType::CloseBrace)
            .is_none()
        {
            root_operation_type_definitions.push(RootOperationTypeDefinition::from_tokens(tokens)?);
        }

        Ok(Self {
            description,
            _schema_span: schema_span,
            directives,
            root_operation_type_definitions,
        })
    }
}

#[derive(Debug)]
pub struct RootOperationTypeDefinition<'a> {
    operation_type: OperationType,
    name: Name<'a>,
}

impl<'a> RootOperationTypeDefinition<'a> {
    pub(crate) fn operation_type(&self) -> &OperationType {
        &self.operation_type
    }

    pub(crate) fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl<'a> FromTokens<'a> for RootOperationTypeDefinition<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let operation_type = tokens.expect_name().and_then(|name| {
            OperationType::from_str(name.as_str()).map_err(|_| ParseError::ExpectedOneOf {
                span: name.span().clone(),
                values: OperationType::POSSIBLE_VALUES,
            })
        })?;
        tokens.expect_punctuator(PunctuatorType::Colon)?;
        let name = tokens.expect_name()?;
        Ok(Self {
            operation_type,
            name,
        })
    }
}
