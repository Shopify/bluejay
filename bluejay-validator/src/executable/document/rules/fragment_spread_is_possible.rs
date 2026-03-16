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
        if !Self::is_composite(parent_type) || !Self::is_composite(fragment_type) {
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

    fn is_composite(t: TypeDefinitionReference<'a, S::TypeDefinition>) -> bool {
        matches!(
            t,
            TypeDefinitionReference::Object(_)
                | TypeDefinitionReference::Interface(_)
                | TypeDefinitionReference::Union(_)
        )
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
        // Iterate over possible types of `a` and check if any is in `b`
        match a {
            TypeDefinitionReference::Object(_) => self.type_contains_name(b, a.name()),
            TypeDefinitionReference::Interface(itd) => self
                .schema_definition
                .get_interface_implementors(itd)
                .any(|otd| self.type_contains_name(b, ObjectTypeDefinition::name(otd))),
            TypeDefinitionReference::Union(utd) => utd
                .union_member_types()
                .iter()
                .any(|member| self.type_contains_name(b, member.name())),
            _ => false,
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
