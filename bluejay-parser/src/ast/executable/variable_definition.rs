use crate::ast::{
    ConstDirectives, ConstValue, FromTokens, ParseError, Tokens, TypeReference, Variable,
};
use crate::lexical_token::PunctuatorType;

#[derive(Debug)]
pub struct VariableDefinition<'a> {
    variable: Variable<'a>,
    r#type: TypeReference<'a>,
    default_value: Option<ConstValue<'a>>,
    directives: ConstDirectives<'a>,
}

impl<'a> FromTokens<'a> for VariableDefinition<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let variable = Variable::from_tokens(tokens)?;
        tokens.expect_punctuator(PunctuatorType::Colon)?;
        let r#type = TypeReference::from_tokens(tokens)?;
        let default_value: Option<ConstValue> =
            if tokens.next_if_punctuator(PunctuatorType::Equals).is_some() {
                Some(ConstValue::from_tokens(tokens)?)
            } else {
                None
            };
        let directives = ConstDirectives::from_tokens(tokens)?;
        Ok(Self {
            variable,
            r#type,
            default_value,
            directives,
        })
    }
}

impl<'a> VariableDefinition<'a> {
    pub fn variable(&self) -> &Variable {
        &self.variable
    }

    pub fn r#type(&self) -> &TypeReference {
        &self.r#type
    }

    pub fn default_value(&self) -> Option<&ConstValue> {
        self.default_value.as_ref()
    }
}

impl<'a> bluejay_core::executable::VariableDefinition for VariableDefinition<'a> {
    type Value = ConstValue<'a>;
    type TypeReference = TypeReference<'a>;
    type Directives = ConstDirectives<'a>;

    fn variable(&self) -> &str {
        self.variable.name()
    }

    fn r#type(&self) -> &Self::TypeReference {
        &self.r#type
    }

    fn directives(&self) -> &Self::Directives {
        &self.directives
    }

    fn default_value(&self) -> Option<&Self::Value> {
        self.default_value.as_ref()
    }
}
