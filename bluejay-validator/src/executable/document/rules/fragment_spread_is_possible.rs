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
    fn spread_is_not_possible(
        &self,
        parent_type: TypeDefinitionReference<'a, S::TypeDefinition>,
        fragment_type: TypeDefinitionReference<'a, S::TypeDefinition>,
    ) -> bool {
        // Fast path: if either type is not a composite type, spread is not applicable
        if !parent_type.is_composite() || !fragment_type.is_composite() {
            return false;
        }

        // Fast path: same type name means definitely possible
        if parent_type.name() == fragment_type.name() {
            return false;
        }

        // Fast path: if both are object types, they can only overlap if they're the same (already checked)
        if matches!(parent_type, TypeDefinitionReference::Object(_))
            && matches!(fragment_type, TypeDefinitionReference::Object(_))
        {
            return true;
        }

        // For mixed cases, check intersection of possible types
        !self.types_have_overlap(parent_type, fragment_type)
    }

    fn type_contains_name(
        &self,
        t: TypeDefinitionReference<'a, S::TypeDefinition>,
        name: &str,
    ) -> bool {
        match t {
            TypeDefinitionReference::Object(_) => t.name() == name,
            TypeDefinitionReference::Interface(itd) => self
                .schema_definition
                .get_interface_implementors(itd)
                .any(|otd| ObjectTypeDefinition::name(otd) == name),
            TypeDefinitionReference::Union(utd) => utd
                .union_member_types()
                .iter()
                .any(|member| member.name() == name),
            _ => false,
        }
    }

    fn types_have_overlap(
        &self,
        a: TypeDefinitionReference<'a, S::TypeDefinition>,
        b: TypeDefinitionReference<'a, S::TypeDefinition>,
    ) -> bool {
        match (a, b) {
            (TypeDefinitionReference::Object(o), other)
            | (other, TypeDefinitionReference::Object(o)) => {
                self.type_contains_name(other, ObjectTypeDefinition::name(o))
            }
            // Both are abstract — collect b's possible types once, then check a's against it
            _ => {
                let b_names: Vec<&str> = self.possible_type_names(b).collect();
                self.possible_type_names(a)
                    .any(|name| b_names.contains(&name))
            }
        }
    }

    fn possible_type_names(
        &self,
        t: TypeDefinitionReference<'a, S::TypeDefinition>,
    ) -> impl Iterator<Item = &'a str> + '_ {
        use itertools::Either;
        match t {
            TypeDefinitionReference::Interface(itd) => Either::Left(
                self.schema_definition
                    .get_interface_implementors(itd)
                    .map(ObjectTypeDefinition::name),
            ),
            TypeDefinitionReference::Union(utd) => Either::Right(Either::Left(
                utd.union_member_types().iter().map(|m| m.name()),
            )),
            _ => Either::Right(Either::Right(std::iter::empty())),
        }
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
