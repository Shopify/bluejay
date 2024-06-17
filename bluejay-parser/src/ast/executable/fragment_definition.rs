use bluejay_core::AsIter;

use crate::ast::executable::{SelectionSet, TypeCondition};
use crate::ast::{FromTokens, IsMatch, ParseError, Tokens, VariableDirectives};
use crate::lexical_token::Name;
use crate::{HasSpan, Span};

#[derive(Debug)]
pub struct FragmentDefinition<'a> {
    name: Name<'a>,
    type_condition: TypeCondition<'a>,
    directives: Option<VariableDirectives<'a>>,
    selection_set: SelectionSet<'a>,
    span: Span,
}

impl<'a> IsMatch<'a> for FragmentDefinition<'a> {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        tokens.peek_name_matches(0, "fragment")
    }
}

impl<'a> FromTokens<'a> for FragmentDefinition<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let fragment_identifier_span = tokens.expect_name_value("fragment")?;
        let name = tokens.expect_name()?;
        if name.as_ref() == TypeCondition::ON {
            // TODO: make this error message better
            return Err(ParseError::UnexpectedToken { span: name.into() });
        }
        let type_condition = TypeCondition::from_tokens(tokens)?;
        let directives = VariableDirectives::from_tokens(tokens)?;
        let selection_set = SelectionSet::from_tokens(tokens)?;
        let span = fragment_identifier_span.merge(selection_set.span());
        Ok(Self {
            name,
            type_condition,
            directives: if directives.len() > 0 {
                Some(directives)
            } else {
                None
            },
            selection_set,
            span,
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

impl<'a> bluejay_core::Indexable for FragmentDefinition<'a> {
    type Id = Span;

    fn id(&self) -> &Self::Id {
        &self.span
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

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }

    fn selection_set(&self) -> &Self::SelectionSet {
        &self.selection_set
    }
}

impl<'a> HasSpan for FragmentDefinition<'a> {
    fn span(&self) -> &Span {
        &self.span
    }
}
