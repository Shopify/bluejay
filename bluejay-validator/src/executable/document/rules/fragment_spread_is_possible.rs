use crate::executable::{
    document::{Error, Path, Rule, Visitor},
    Cache,
};
use bluejay_core::definition::{
    ObjectTypeDefinition, SchemaDefinition, TypeDefinitionReference, UnionMemberType,
    UnionTypeDefinition,
};
use bluejay_core::executable::{
    ExecutableDocument, FragmentDefinition, FragmentSpread, InlineFragment,
};
use bluejay_core::AsIter;
use std::collections::HashSet;

pub struct FragmentSpreadIsPossible<'a, E: ExecutableDocument, S: SchemaDefinition> {
    errors: Vec<Error<'a, E, S>>,
    cache: &'a Cache<'a, E, S>,
    schema_definition: &'a S,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Visitor<'a, E, S>
    for FragmentSpreadIsPossible<'a, E, S>
{
    fn new(_: &'a E, schema_definition: &'a S, cache: &'a Cache<'a, E, S>) -> Self {
        Self {
            errors: Vec::new(),
            cache,
            schema_definition,
        }
    }

    fn visit_fragment_spread(
        &mut self,
        fragment_spread: &'a <E as ExecutableDocument>::FragmentSpread,
        parent_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        _path: &Path<'a, E>,
    ) {
        if let Some(fragment_definition) = self.cache.fragment_definition(fragment_spread.name()) {
            if let Some(fragment_type) = self
                .schema_definition
                .get_type_definition(fragment_definition.type_condition())
            {
                if self.spread_is_not_possible(parent_type, fragment_type) {
                    self.errors.push(Error::FragmentSpreadIsNotPossible {
                        fragment_spread,
                        parent_type,
                    });
                }
            }
        }
    }

    fn visit_inline_fragment(
        &mut self,
        inline_fragment: &'a <E as ExecutableDocument>::InlineFragment,
        parent_type: TypeDefinitionReference<'a, S::TypeDefinition>,
    ) {
        if let Some(type_condition) = inline_fragment.type_condition() {
            if let Some(fragment_type) = self.schema_definition.get_type_definition(type_condition)
            {
                if self.spread_is_not_possible(parent_type, fragment_type) {
                    self.errors.push(Error::InlineFragmentSpreadIsNotPossible {
                        inline_fragment,
                        parent_type,
                    });
                }
            }
        }
    }
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> FragmentSpreadIsPossible<'a, E, S> {
    fn get_possible_types(
        &self,
        t: TypeDefinitionReference<'a, S::TypeDefinition>,
    ) -> Option<HashSet<&'a str>> {
        match t {
            TypeDefinitionReference::Object(_) => Some(HashSet::from([t.name()])),
            TypeDefinitionReference::Interface(itd) => Some(HashSet::from_iter(
                self.schema_definition
                    .get_interface_implementors(itd)
                    .map(ObjectTypeDefinition::name),
            )),
            TypeDefinitionReference::Union(utd) => Some(HashSet::from_iter(
                utd.union_member_types()
                    .iter()
                    .map(|union_member| union_member.name()),
            )),
            TypeDefinitionReference::BuiltinScalar(_)
            | TypeDefinitionReference::CustomScalar(_)
            | TypeDefinitionReference::Enum(_)
            | TypeDefinitionReference::InputObject(_) => None,
        }
    }

    fn spread_is_not_possible(
        &self,
        parent_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        fragment_type: TypeDefinitionReference<'a, S::TypeDefinition>,
    ) -> bool {
        let parent_type_possible_types = self.get_possible_types(parent_type);
        let fragment_possible_types = self.get_possible_types(fragment_type);

        matches!(
            (parent_type_possible_types, fragment_possible_types),
            (Some(parent_type_possible_types), Some(fragment_possible_types)) if parent_type_possible_types
                .intersection(&fragment_possible_types)
                .next()
                .is_none(),
        )
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for FragmentSpreadIsPossible<'a, E, S>
{
    type Error = Error<'a, E, S>;
    type Errors = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_errors(self) -> Self::Errors {
        self.errors.into_iter()
    }
}
