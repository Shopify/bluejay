use crate::ast::definition::{Context, UnionMemberType};
use crate::ast::{DepthLimiter, FromTokens, ParseError, Tokens};
use crate::lexical_token::PunctuatorType;
use bluejay_core::definition::UnionMemberTypes as CoreUnionMemberTypes;
use bluejay_core::AsIter;

#[derive(Debug)]
pub struct UnionMemberTypes<'a, C: Context> {
    union_member_types: Vec<UnionMemberType<'a, C>>,
}

impl<'a, C: Context> AsIter for UnionMemberTypes<'a, C> {
    type Item = UnionMemberType<'a, C>;
    type Iterator<'b>
        = std::slice::Iter<'b, Self::Item>
    where
        'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.union_member_types.iter()
    }
}

impl<'a, C: Context> CoreUnionMemberTypes for UnionMemberTypes<'a, C> {
    type UnionMemberType = UnionMemberType<'a, C>;
}

impl<'a, C: Context> FromTokens<'a> for UnionMemberTypes<'a, C> {
    fn from_tokens(
        tokens: &mut impl Tokens<'a>,
        depth_limiter: DepthLimiter,
    ) -> Result<Self, ParseError> {
        tokens.next_if_punctuator(PunctuatorType::Pipe);
        let mut union_member_types =
            vec![UnionMemberType::from_tokens(tokens, depth_limiter.bump()?)?];
        while tokens.next_if_punctuator(PunctuatorType::Pipe).is_some() {
            union_member_types.push(UnionMemberType::from_tokens(tokens, depth_limiter.bump()?)?);
        }
        Ok(Self { union_member_types })
    }
}
