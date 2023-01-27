use crate::lexical_token::PunctuatorType;
use crate::ast::executable::{TypeCondition, SelectionSet};
use crate::ast::{Tokens, ParseError, FromTokens, TryFromTokens, VariableDirectives, IsMatch};

#[derive(Debug)]
pub struct InlineFragment<'a> {
    type_condition: Option<TypeCondition<'a>>,
    directives: VariableDirectives<'a>,
    selection_set: SelectionSet<'a>,
}

impl<'a> FromTokens<'a> for InlineFragment<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        tokens.expect_punctuator(PunctuatorType::Ellipse)?;
        let type_condition = TypeCondition::try_from_tokens(tokens).transpose()?;
        let directives = VariableDirectives::from_tokens(tokens)?;
        let selection_set = SelectionSet::from_tokens(tokens)?;
        Ok(Self { type_condition, directives, selection_set })
    }
}

impl<'a> IsMatch<'a> for InlineFragment<'a> {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        tokens.peek_punctuator_matches(0, PunctuatorType::Ellipse) && tokens.peek_name(1).map(|n| n.as_ref() == TypeCondition::ON).unwrap_or(true)
    }
}

impl<'a> InlineFragment<'a> {
    pub fn type_condition(&self) -> Option<&str> {
        self.type_condition.as_ref().map(|tc| tc.named_type().as_ref())
    }

    pub fn selection_set(&self) -> &SelectionSet<'a> {
        &self.selection_set
    }
}

impl<'a> bluejay_core::executable::InlineFragment for InlineFragment<'a> {
    type Directives = VariableDirectives<'a>;
    type SelectionSet = SelectionSet<'a>;

    fn type_condition(&self) -> Option<&str> {
        self.type_condition.as_ref().map(|tc| tc.named_type().as_ref())
    }

    fn directives(&self) -> &Self::Directives {
        &self.directives
    }

    fn selection_set(&self) -> &Self::SelectionSet {
        &self.selection_set
    }
}
