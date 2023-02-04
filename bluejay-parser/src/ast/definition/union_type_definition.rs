use crate::ast::definition::UnionMemberTypes;
use crate::ast::{ConstDirectives, FromTokens, ParseError, Tokens, TryFromTokens};
use crate::lexical_token::{Name, PunctuatorType, StringValue};
use bluejay_core::definition::UnionTypeDefinition as CoreUnionTypeDefinition;

#[derive(Debug)]
pub struct UnionTypeDefinition<'a> {
    description: Option<StringValue>,
    name: Name<'a>,
    directives: Option<ConstDirectives<'a>>,
    member_types: UnionMemberTypes<'a>,
}

impl<'a> CoreUnionTypeDefinition for UnionTypeDefinition<'a> {
    type UnionMemberTypes = UnionMemberTypes<'a>;
    type Directives = ConstDirectives<'a>;

    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(AsRef::as_ref)
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }

    fn directives(&self) -> Option<&Self::Directives> {
        self.directives.as_ref()
    }

    fn union_member_types(&self) -> &Self::UnionMemberTypes {
        &self.member_types
    }
}

impl<'a> UnionTypeDefinition<'a> {
    pub(crate) const UNION_IDENTIFIER: &'static str = "union";
}

impl<'a> FromTokens<'a> for UnionTypeDefinition<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        let description = tokens.next_if_string_value();
        tokens.expect_name_value(Self::UNION_IDENTIFIER)?;
        let name = tokens.expect_name()?;
        let directives = ConstDirectives::try_from_tokens(tokens).transpose()?;
        tokens.expect_punctuator(PunctuatorType::Equals)?;
        let member_types = UnionMemberTypes::from_tokens(tokens)?;
        Ok(Self {
            description,
            name,
            directives,
            member_types,
        })
    }
}

impl<'a> AsRef<UnionTypeDefinition<'a>> for UnionTypeDefinition<'a> {
    fn as_ref(&self) -> &UnionTypeDefinition<'a> {
        self
    }
}
