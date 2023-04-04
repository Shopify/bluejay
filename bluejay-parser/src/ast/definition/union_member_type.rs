use crate::ast::definition::ObjectTypeDefinition;
use crate::ast::{FromTokens, ParseError, Tokens};
use crate::lexical_token::Name;
use bluejay_core::definition::UnionMemberType as CoreUnionMemberType;
use once_cell::sync::OnceCell;

#[derive(Debug)]
pub struct UnionMemberType<'a> {
    name: Name<'a>,
    r#type: OnceCell<&'a ObjectTypeDefinition<'a>>,
}

impl<'a> CoreUnionMemberType for UnionMemberType<'a> {
    type ObjectTypeDefinition = ObjectTypeDefinition<'a>;

    fn member_type(&self) -> &Self::ObjectTypeDefinition {
        self.r#type.get().unwrap()
    }
}

impl<'a> UnionMemberType<'a> {
    pub(crate) fn name(&self) -> &Name<'a> {
        &self.name
    }

    pub(crate) fn set_type_reference(
        &self,
        type_reference: &'a ObjectTypeDefinition<'a>,
    ) -> Result<(), &'a ObjectTypeDefinition<'a>> {
        self.r#type.set(type_reference)
    }
}

impl<'a> FromTokens<'a> for UnionMemberType<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        tokens.expect_name().map(|name| Self {
            name,
            r#type: OnceCell::new(),
        })
    }
}
