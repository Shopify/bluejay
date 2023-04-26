use crate::ast::definition::{Context, ObjectTypeDefinition};
use crate::ast::{FromTokens, ParseError, Tokens};
use crate::lexical_token::Name;
use bluejay_core::definition::UnionMemberType as CoreUnionMemberType;
use once_cell::sync::OnceCell;

#[derive(Debug)]
pub struct UnionMemberType<'a, C: Context> {
    name: Name<'a>,
    r#type: OnceCell<&'a ObjectTypeDefinition<'a, C>>,
}

impl<'a, C: Context> CoreUnionMemberType for UnionMemberType<'a, C> {
    type ObjectTypeDefinition = ObjectTypeDefinition<'a, C>;

    fn member_type(&self) -> &Self::ObjectTypeDefinition {
        self.r#type.get().unwrap()
    }
}

impl<'a, C: Context> UnionMemberType<'a, C> {
    pub(crate) fn name(&self) -> &Name<'a> {
        &self.name
    }

    pub(crate) fn set_type_reference(
        &self,
        type_reference: &'a ObjectTypeDefinition<'a, C>,
    ) -> Result<(), &'a ObjectTypeDefinition<'a, C>> {
        self.r#type.set(type_reference)
    }
}

impl<'a, C: Context> FromTokens<'a> for UnionMemberType<'a, C> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        tokens.expect_name().map(|name| Self {
            name,
            r#type: OnceCell::new(),
        })
    }
}
