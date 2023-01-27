use crate::ast::executable::{OperationDefinition, FragmentDefinition};
use crate::ast::{Tokens, ParseError, FromTokens, IsMatch};

pub type ExecutableDefinition<'a> = bluejay_core::executable::ExecutableDefinition<OperationDefinition<'a>, FragmentDefinition<'a>>;

impl<'a> FromTokens<'a> for ExecutableDefinition<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        if OperationDefinition::is_match(tokens) {
            OperationDefinition::from_tokens(tokens).map(Self::Operation)
        } else if FragmentDefinition::is_match(tokens) {
            FragmentDefinition::from_tokens(tokens).map(Self::Fragment)
        } else {
            Err(tokens.unexpected_token())
        }
    }
}

impl<'a> IsMatch<'a> for ExecutableDefinition<'a> {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        OperationDefinition::is_match(tokens) || FragmentDefinition::is_match(tokens)
    }
}
