use crate::executable::{Error, Rule, Visitor};
use bluejay_core::definition::{
    ObjectTypeDefinition, SchemaDefinition, TypeDefinitionReference,
    TypeDefinitionReferenceFromAbstract, UnionMemberType, UnionTypeDefinition,
};
use bluejay_core::executable::{
    ExecutableDocument, FragmentDefinition, FragmentSpread, InlineFragment,
};
use bluejay_core::AsIter;
use std::collections::{HashMap, HashSet};

pub struct FragmentSpreadIsPossible<'a, E: ExecutableDocument, S: SchemaDefinition> {
    errors: Vec<Error<'a, E, S>>,
    indexed_fragment_definitions: HashMap<&'a str, &'a E::FragmentDefinition>,
    schema_definition: &'a S,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Visitor<'a, E, S>
    for FragmentSpreadIsPossible<'a, E, S>
{
    fn visit_fragment_spread(
        &mut self,
        fragment_spread: &'a <E as ExecutableDocument>::FragmentSpread,
        parent_type: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
    ) {
        if let Some(fragment_definition) = self
            .indexed_fragment_definitions
            .get(fragment_spread.name())
        {
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
        parent_type: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
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
        t: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
    ) -> Option<HashSet<&'a str>> {
        match t {
            TypeDefinitionReference::ObjectType(_) => Some(HashSet::from([t.name()])),
            TypeDefinitionReference::InterfaceType(itd) => Some(HashSet::from_iter(
                self.schema_definition
                    .get_interface_implementors(itd)
                    .map(ObjectTypeDefinition::name),
            )),
            TypeDefinitionReference::UnionType(utd) => Some(HashSet::from_iter(
                utd.union_member_types()
                    .iter()
                    .map(|union_member| union_member.member_type().name()),
            )),
            TypeDefinitionReference::BuiltinScalarType(_)
            | TypeDefinitionReference::CustomScalarType(_)
            | TypeDefinitionReference::EnumType(_)
            | TypeDefinitionReference::InputObjectType(_) => None,
        }
    }

    fn spread_is_not_possible(
        &self,
        parent_type: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
        fragment_type: TypeDefinitionReferenceFromAbstract<'a, S::TypeDefinitionReference>,
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

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> IntoIterator
    for FragmentSpreadIsPossible<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for FragmentSpreadIsPossible<'a, E, S>
{
    fn new(executable_document: &'a E, schema_definition: &'a S) -> Self {
        Self {
            errors: Vec::new(),
            indexed_fragment_definitions: HashMap::from_iter(
                executable_document
                    .fragment_definitions()
                    .as_ref()
                    .iter()
                    .map(|fragment_definition| (fragment_definition.name(), fragment_definition)),
            ),
            schema_definition,
        }
    }
}
