use crate::ast::{ParseError, Tokens, FromTokens, IsMatch, VariableDirectives, TryFromTokens};
use crate::ast::executable::{VariableDefinitions, SelectionSet};
use crate::lexical_token::Name;
use bluejay_core::OperationType;

pub type OperationDefinition<'a> = bluejay_core::executable::OperationDefinition<ExplicitOperationDefinition<'a>, ImplicitOperationDefinition<'a>>;

impl<'a> FromTokens<'a> for OperationDefinition<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        if let Some(operation_type) = OperationType::try_from_tokens(tokens).transpose()? {
            let name = tokens.next_if_name();
            let variable_definitions = VariableDefinitions::try_from_tokens(tokens).transpose()?;
            let directives = VariableDirectives::from_tokens(tokens)?;
            let selection_set = SelectionSet::from_tokens(tokens)?;
            Ok(Self::Explicit(ExplicitOperationDefinition { operation_type, name, variable_definitions, directives, selection_set }))
        } else if let Some(selection_set) = SelectionSet::try_from_tokens(tokens).transpose()? {
            Ok(Self::Implicit(ImplicitOperationDefinition { selection_set }))
        } else {
            Err(tokens.unexpected_token())
        }
    }
}

impl<'a> IsMatch<'a> for OperationDefinition<'a> {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        OperationType::is_match(tokens) || SelectionSet::is_match(tokens)
    }
}

#[derive(Debug)]
pub struct ExplicitOperationDefinition<'a> {
    operation_type: OperationType,
    name: Option<Name<'a>>,
    variable_definitions: Option<VariableDefinitions<'a>>,
    directives: VariableDirectives<'a>,
    selection_set: SelectionSet<'a>,
}

impl<'a> bluejay_core::executable::ExplicitOperationDefinition for ExplicitOperationDefinition<'a> {
    type VariableDefinitions = VariableDefinitions<'a>;
    type Directives = VariableDirectives<'a>;
    type SelectionSet = SelectionSet<'a>;

    fn operation_type(&self) -> &OperationType {
        &self.operation_type
    }

    fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|name| name.as_ref())
    }

    fn variable_definitions(&self) -> Option<&Self::VariableDefinitions> {
        self.variable_definitions.as_ref()
    }

    fn directives(&self) -> &Self::Directives {
        &self.directives
    }

    fn selection_set(&self) -> &Self::SelectionSet {
        &self.selection_set
    }
}

#[derive(Debug)]
pub struct ImplicitOperationDefinition<'a> {
    selection_set: SelectionSet<'a>,
}

impl<'a> bluejay_core::executable::ImplicitOperationDefinition for ImplicitOperationDefinition<'a> {
    type SelectionSet = SelectionSet<'a>;

    fn selection_set(&self) -> &Self::SelectionSet {
        &self.selection_set
    }
}
