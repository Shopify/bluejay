use crate::ast::executable::SelectionSet;
use crate::ast::{
    FromTokens, IsMatch, ParseError, Tokens, TryFromTokens, VariableArguments, VariableDirectives,
};
use crate::lexical_token::{Name, PunctuatorType};

#[derive(Debug)]
pub struct Field<'a> {
    alias: Option<Name<'a>>,
    name: Name<'a>,
    arguments: Option<VariableArguments<'a>>,
    directives: VariableDirectives<'a>,
    selection_set: Option<SelectionSet<'a>>,
}

impl<'a> FromTokens<'a> for Field<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let has_alias = tokens.peek_punctuator_matches(1, PunctuatorType::Colon);
        let (alias, name) = if has_alias {
            let alias = Some(tokens.expect_name()?);
            tokens.expect_punctuator(PunctuatorType::Colon)?;
            let name = tokens.expect_name()?;
            (alias, name)
        } else {
            (None, tokens.expect_name()?)
        };
        let arguments = VariableArguments::try_from_tokens(tokens).transpose()?;
        let directives = VariableDirectives::from_tokens(tokens)?;
        let selection_set = SelectionSet::try_from_tokens(tokens).transpose()?;
        Ok(Self {
            alias,
            name,
            arguments,
            directives,
            selection_set,
        })
    }
}

impl<'a> IsMatch<'a> for Field<'a> {
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        tokens.peek_name(0).is_some()
    }
}

impl<'a> Field<'a> {
    pub fn response_key(&self) -> &str {
        if let Some(alias) = &self.alias {
            alias.as_ref()
        } else {
            self.name.as_ref()
        }
    }

    pub fn name(&self) -> &Name<'a> {
        &self.name
    }

    pub fn arguments(&self) -> Option<&VariableArguments> {
        self.arguments.as_ref()
    }

    pub fn selection_set(&self) -> Option<&SelectionSet> {
        self.selection_set.as_ref()
    }
}

impl<'a> bluejay_core::executable::Field for Field<'a> {
    type Arguments = VariableArguments<'a>;
    type Directives = VariableDirectives<'a>;
    type SelectionSet = SelectionSet<'a>;

    fn alias(&self) -> Option<&str> {
        self.alias.as_ref().map(|name| name.as_ref())
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn arguments(&self) -> Option<&Self::Arguments> {
        self.arguments.as_ref()
    }

    fn directives(&self) -> &Self::Directives {
        &self.directives
    }

    fn selection_set(&self) -> Option<&Self::SelectionSet> {
        self.selection_set.as_ref()
    }
}
