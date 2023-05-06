use crate::ast::executable::{FragmentDefinition, OperationDefinition};
use crate::ast::{FromTokens, IsMatch, ParseError, Tokens};

#[derive(Debug)]
pub enum ExecutableDefinition<'a> {
    Operation(OperationDefinition<'a>),
    Fragment(FragmentDefinition<'a>),
}

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
