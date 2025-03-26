use crate::ast::executable::SelectionSet;
use crate::ast::{
    DepthLimiter, FromTokens, IsMatch, ParseError, Tokens, TryFromTokens, VariableArguments,
    VariableDirectives,
};
use crate::lexical_token::{Name, PunctuatorType};
use crate::{HasSpan, Span};
use std::cmp::{Eq, PartialEq};
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct Field<'a> {
    alias: Option<Name<'a>>,
    name: Name<'a>,
    arguments: Option<VariableArguments<'a>>,
    directives: Option<VariableDirectives<'a>>,
    selection_set: Option<SelectionSet<'a>>,
    span: Span,
}

impl<'a> FromTokens<'a> for Field<'a> {
    #[inline]
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        let has_alias = tokens.peek_punctuator_matches(1, PunctuatorType::Colon);
        let (alias, name) = if has_alias {
            let alias = Some(tokens.expect_name()?);
            tokens.expect_punctuator(PunctuatorType::Colon)?;
            let name = tokens.expect_name()?;
            (alias, name)
        } else {
            (None, tokens.expect_name()?)
        };
        let arguments =
            VariableArguments::try_from_tokens(tokens, depth_limiter.bump()?).transpose()?;
        let directives =
            VariableDirectives::try_from_tokens(tokens, depth_limiter.bump()?).transpose()?;
        let selection_set =
            SelectionSet::try_from_tokens(tokens, depth_limiter.bump()?).transpose()?;
        let start_span = alias.as_ref().unwrap_or(&name).span();
        let directives_span = directives.as_ref().and_then(|directives| directives.span());
        let end_span = if let Some(selection_set) = &selection_set {
            selection_set.span()
        } else if let Some(directive_span) = directives_span {
            directive_span
        } else if let Some(arguments) = &arguments {
            arguments.span()
        } else {
            name.span()
        };
        let span = start_span.merge(end_span);
        Ok(Self {
            alias,
            name,
            arguments,
            directives,
            selection_set,
            span,
        })
    }
}

impl<'a> IsMatch<'a> for Field<'a> {
    #[inline]
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

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }

    fn selection_set(&self) -> Option<&Self::SelectionSet> {
        self.selection_set.as_ref()
    }
}

impl HasSpan for Field<'_> {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl Hash for Field<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.span().hash(state);
    }
}

impl PartialEq for Field<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.span() == other.span()
    }
}

impl Eq for Field<'_> {}
