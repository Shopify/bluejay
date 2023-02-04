use crate::ast::definition::UnionMemberType;
use crate::ast::{FromTokens, ParseError, Tokens};
use crate::lexical_token::PunctuatorType;
use bluejay_core::definition::UnionMemberTypes as CoreUnionMemberTypes;
use bluejay_core::AsIter;

#[derive(Debug)]
pub struct UnionMemberTypes<'a> {
    union_member_types: Vec<UnionMemberType<'a>>,
}

impl<'a> AsIter for UnionMemberTypes<'a> {
    type Item = UnionMemberType<'a>;
    type Iterator<'b> = std::slice::Iter<'b, Self::Item> where 'a: 'b;

    fn iter(&self) -> Self::Iterator<'_> {
        self.union_member_types.iter()
    }
}

impl<'a> CoreUnionMemberTypes for UnionMemberTypes<'a> {
    type UnionMemberType = UnionMemberType<'a>;
}

impl<'a> FromTokens<'a> for UnionMemberTypes<'a> {
    fn from_tokens(tokens: &mut impl Tokens<'a>) -> Result<Self, ParseError> {
        tokens.next_if_punctuator(PunctuatorType::Pipe);
        let mut union_member_types = vec![UnionMemberType::from_tokens(tokens)?];
        while tokens.next_if_punctuator(PunctuatorType::Pipe).is_some() {
            union_member_types.push(UnionMemberType::from_tokens(tokens)?);
        }
        Ok(Self { union_member_types })
    }
}
