use crate::ast::definition::{
    Context, Directives, FieldsDefinition, SchemaDefinition, UnionMemberTypes,
};
use crate::ast::{ConstDirectives, DepthLimiter, FromTokens, ParseError, Tokens, TryFromTokens};
use crate::lexical_token::{Name, PunctuatorType, StringValue};
use crate::{HasSpan, Span};
use bluejay_core::definition::{HasDirectives, UnionTypeDefinition as CoreUnionTypeDefinition};

#[derive(Debug)]
pub struct UnionTypeDefinition<'a, C: Context> {
    description: Option<StringValue<'a>>,
    name: Name<'a>,
    directives: Option<Directives<'a, C>>,
    member_types: UnionMemberTypes<'a, C>,
    fields_definition: FieldsDefinition<'a, C>,
    span: Span,
}

impl<'a, C: Context> CoreUnionTypeDefinition for UnionTypeDefinition<'a, C> {
    type SchemaDefinition = SchemaDefinition<'a, C>;

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn union_member_types(&self) -> &UnionMemberTypes<'a, C> {
        &self.member_types
    }

    fn fields_definition(&self) -> &FieldsDefinition<'a, C> {
        &self.fields_definition
    }
}

impl<'a, C: Context> UnionTypeDefinition<'a, C> {
    pub(crate) const UNION_IDENTIFIER: &'static str = "union";

    pub(crate) fn name(&self) -> &Name<'a> {
        &self.name
    }

    pub fn name_token(&self) -> &Name<'a> {
        &self.name
    }

    pub fn description_token(&self) -> Option<&StringValue<'a>> {
        self.description.as_ref()
    }
}

impl<'a, C: Context> FromTokens<'a> for UnionTypeDefinition<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();
        let union_identifier_span = tokens.expect_name_value(Self::UNION_IDENTIFIER)?;
        let name = tokens.expect_name()?;
        let directives = ConstDirectives::try_from_tokens(tokens, depth_limiter.bump()?)?;
        // Optional in the grammar. `SchemaDefinition::try_from` rejects an empty type.
        let member_types = if tokens.next_if_punctuator(PunctuatorType::Equals).is_some() {
            UnionMemberTypes::from_tokens(tokens, depth_limiter.bump()?)?
        } else {
            UnionMemberTypes::empty()
        };
        let start = description
            .as_ref()
            .map_or(union_identifier_span, |description| *description.span());
        let end = member_types
            .span()
            .or_else(|| {
                directives
                    .as_ref()
                    .and_then(|directives| directives.span().copied())
            })
            .unwrap_or(*name.span());
        Ok(Self {
            description,
            name,
            directives: directives.map(Directives::from),
            member_types,
            fields_definition: FieldsDefinition::__typename(),
            span: start.merge(&end),
        })
    }
}

impl<C: Context> HasSpan for UnionTypeDefinition<'_, C> {
    fn span(&self) -> &Span {
        &self.span
    }
}

impl<'a, C: Context> HasDirectives for UnionTypeDefinition<'a, C> {
    type Directives = Directives<'a, C>;

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }
}
