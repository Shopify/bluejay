use crate::ast::executable::{SelectionSet, TypeCondition};
use crate::ast::{FromTokens, IsMatch, ParseError, Tokens, VariableDirectives};
use crate::lexical_token::Name;

#[derive(Debug)]
pub struct FragmentDefinition<'a> {
    name: Name<'a>,
    type_condition: TypeCondition<'a>,
    directives: VariableDirectives<'a>,
    selection_set: SelectionSet<'a>,
}

impl<'a> IsMatch<'a> for FragmentDefinition<'a> {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        tokens.peek_name_matches(0, "fragment")
    }
}

impl<'a> FromTokens<'a> for FragmentDefinition<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        tokens.expect_name_value("fragment")?;
        let name = tokens.expect_name()?;
        if name.as_ref() == TypeCondition::ON {
            // TODO: make this error message better
            return Err(ParseError::UnexpectedToken { span: name.into() });
        }
        let type_condition = TypeCondition::from_tokens(tokens)?;
        let directives = VariableDirectives::from_tokens(tokens)?;
        let selection_set = SelectionSet::from_tokens(tokens)?;
        Ok(Self {
            name,
            type_condition,
            directives,
            selection_set,
        })
    }
}

impl<'a> FragmentDefinition<'a> {
    pub fn name(&self) -> &Name<'a> {
        &self.name
    }

    pub fn type_condition(&self) -> &TypeCondition<'a> {
        &self.type_condition
    }

    pub fn selection_set(&self) -> &SelectionSet {
        &self.selection_set
    }
}

impl<'a> bluejay_core::executable::FragmentDefinition for FragmentDefinition<'a> {
    type Directives = VariableDirectives<'a>;
    type SelectionSet = SelectionSet<'a>;

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn type_condition(&self) -> &str {
        self.type_condition.named_type().as_ref()
    }

    fn directives(&self) -> &Self::Directives {
        &self.directives
    }

    fn selection_set(&self) -> &Self::SelectionSet {
        &self.selection_set
    }
}
