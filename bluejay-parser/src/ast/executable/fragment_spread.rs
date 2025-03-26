use crate::ast::executable::TypeCondition;
use crate::ast::try_from_tokens::TryFromTokens;
use crate::ast::{DepthLimiter, FromTokens, IsMatch, ParseError, Tokens, VariableDirectives};
use crate::lexical_token::{Name, PunctuatorType};
use crate::{HasSpan, Span};

#[derive(Debug)]
pub struct FragmentSpread<'a> {
    name: Name<'a>,
    directives: Option<VariableDirectives<'a>>,
    span: Span,
}

impl<'a> FromTokens<'a> for FragmentSpread<'a> {
    #[inline]
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        let ellipse_span = tokens.expect_punctuator(PunctuatorType::Ellipse)?;
        let name = tokens.expect_name()?;
        assert_ne!(TypeCondition::ON, name.as_ref());
        let directives =
            VariableDirectives::try_from_tokens(tokens, depth_limiter.bump()?).transpose()?;
        let span = ellipse_span.merge(name.span());
        Ok(Self {
            name,
            directives,
            span,
        })
    }
}

impl<'a> IsMatch<'a> for FragmentSpread<'a> {
    #[inline]
    fn is_match(tokens: &mut impl Tokens<'a>) -> bool {
        tokens.peek_punctuator_matches(0, PunctuatorType::Ellipse)
            && tokens
                .peek_name(1)
                .map(|n| n.as_ref() != TypeCondition::ON)
                .unwrap_or(false)
    }
}

impl<'a> FragmentSpread<'a> {
    pub fn name(&self) -> &Name<'a> {
        &self.name
    }
}

impl<'a> bluejay_core::executable::FragmentSpread for FragmentSpread<'a> {
    type Directives = VariableDirectives<'a>;

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}

impl HasSpan for FragmentSpread<'_> {
    fn span(&self) -> &Span {
        &self.span
    }
}
