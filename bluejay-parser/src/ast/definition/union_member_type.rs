use std::marker::PhantomData;

use crate::ast::definition::{Context, ObjectTypeDefinition, SchemaDefinition};
use crate::ast::{DepthLimiter, FromTokens, ParseError, Tokens};
use crate::lexical_token::Name;
use bluejay_core::definition::{
    SchemaDefinition as CoreSchemaDefinition, UnionMemberType as CoreUnionMemberType,
};

#[derive(Debug)]
pub struct UnionMemberType<'a, C: Context + 'a> {
    name: Name<'a>,
    context: PhantomData<C>,
}

impl<'a, C: Context + 'a> CoreUnionMemberType for UnionMemberType<'a, C> {
    type SchemaDefinition = SchemaDefinition<'a, C>;

    fn member_type<'b>(
        &'b self,
        schema_definition: &'b SchemaDefinition<'a, C>,
    ) -> &'b ObjectTypeDefinition<'a, C> {
        schema_definition
            .get_type_definition(self.name.as_str())
            .unwrap()
            .as_object()
            .unwrap()
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }
}

impl<'a, C: Context> UnionMemberType<'a, C> {
    pub(crate) fn name(&self) -> &Name<'a> {
        &self.name
    }
}

impl<'a, C: Context> FromTokens<'a> for UnionMemberType<'a, C> {
    fn from_tokens(tokens: &mut impl Tokens<'a>, _: DepthLimiter) -> Result<Self, ParseError> {
        tokens.expect_name().map(|name| Self {
            name,
            context: PhantomData,
        })
    }
}
