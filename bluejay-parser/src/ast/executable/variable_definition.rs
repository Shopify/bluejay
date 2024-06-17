use bluejay_core::AsIter;

use crate::ast::{
    executable::VariableType, ConstDirectives, ConstValue, FromTokens, ParseError, Tokens, Variable,
};
use crate::lexical_token::PunctuatorType;

#[derive(Debug)]
pub struct VariableDefinition<'a> {
    variable: Variable<'a>,
    r#type: VariableType<'a>,
    default_value: Option<ConstValue<'a>>,
    directives: Option<ConstDirectives<'a>>,
}

impl<'a> FromTokens<'a> for VariableDefinition<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let variable = Variable::from_tokens(tokens)?;
        tokens.expect_punctuator(PunctuatorType::Colon)?;
        let r#type = VariableType::from_tokens(tokens)?;
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
            directives: if directives.len() > 0 { Some(directives) } else { None },
        })
    }
}

impl<'a> VariableDefinition<'a> {
    pub fn variable(&self) -> &Variable {
        &self.variable
    }

    pub fn r#type(&self) -> &VariableType {
        &self.r#type
    }

    pub fn default_value(&self) -> Option<&ConstValue> {
        self.default_value.as_ref()
    }
}

impl<'a> bluejay_core::executable::VariableDefinition for VariableDefinition<'a> {
    type Value = ConstValue<'a>;
    type VariableType = VariableType<'a>;
    type Directives = ConstDirectives<'a>;

    fn variable(&self) -> &str {
        self.variable.name()
    }

    fn r#type(&self) -> &Self::VariableType {
        &self.r#type
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }

    fn default_value(&self) -> Option<&Self::Value> {
        self.default_value.as_ref()
    }
}
