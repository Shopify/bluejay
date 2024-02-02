use crate::validation::Error;
use bluejay_core::{
    definition::{
        SchemaDefinition, TypeDefinitionReference, UnionMemberTypes, UnionTypeDefinition,
    },
    executable::{ExecutableDocument, Field, InlineFragment, Selection, SelectionReference},
    AsIter,
};
use bluejay_validator::executable::{Cache, Rule, Visitor};

pub(crate) struct SelectionsAreValid<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> {
    errors: Vec<Error<'a, E, S>>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Visitor<'a, E, S>
    for SelectionsAreValid<'a, E, S>
{
    fn new(_: &'a E, _: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        Self { errors: Vec::new() }
    }

    fn visit_selection_set(
        &mut self,
        selection_set: &'a E::SelectionSet,
        ty: TypeDefinitionReference<'a, S::TypeDefinition>,
    ) {
        match ty {
            TypeDefinitionReference::Object(_) => self.visit_object_selection_set(selection_set),
            TypeDefinitionReference::Union(utd) => {
                self.visit_union_selection_set(selection_set, utd)
            }
            TypeDefinitionReference::Interface(_) => {
                self.visit_interface_selection_set(selection_set)
            }
            _ => {}
        }
    }
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> SelectionsAreValid<'a, E, S> {
    fn visit_object_selection_set(&mut self, selection_set: &'a E::SelectionSet) {
        self.validate_fragment_spreads(selection_set);

        self.errors.extend(
            selection_set
                .iter()
                .filter_map(|selection| match selection.as_ref() {
                    SelectionReference::InlineFragment(inline_fragment) => {
                        Some(Error::InlineFragmentOnObject { inline_fragment })
                    }
                    SelectionReference::Field(_) | SelectionReference::FragmentSpread(_) => None,
                }),
        );
    }

    fn visit_interface_selection_set(&mut self, selection_set: &'a E::SelectionSet) {
        self.validate_fragment_spreads(selection_set);

        self.errors.extend(
            selection_set
                .iter()
                .filter_map(|selection| match selection.as_ref() {
                    SelectionReference::InlineFragment(inline_fragment) => {
                        Some(Error::InlineFragmentOnInterface { inline_fragment })
                    }
                    SelectionReference::Field(_) | SelectionReference::FragmentSpread(_) => None,
                }),
        );
    }

    fn visit_union_selection_set(
        &mut self,
        selection_set: &'a E::SelectionSet,
        union_type_definition: &'a S::UnionTypeDefinition,
    ) {
        if self.validate_fragment_spreads(selection_set) {
            return;
        }

        let (typename_selection, other_field_selections) = self.typename_selection(selection_set);

        if typename_selection.is_none() {
            self.errors
                .push(Error::NoTypenameSelectionOnUnion { selection_set });
        }

        self.errors.extend(
            other_field_selections
                .into_iter()
                .map(|field| Error::FieldSelectionOnUnion { field }),
        );

        let inline_fragments =
            selection_set
                .iter()
                .filter_map(|selection| match selection.as_ref() {
                    SelectionReference::InlineFragment(inline_fragment) => Some(inline_fragment),
                    SelectionReference::Field(_) | SelectionReference::FragmentSpread(_) => None,
                });

        let (targets_member_type, does_not_target_member_type): (Vec<_>, Vec<_>) = inline_fragments
            .partition(|inline_fragment| {
                inline_fragment.type_condition().map_or(false, |name| {
                    union_type_definition
                        .union_member_types()
                        .contains_type(name)
                })
            });

        self.errors.extend(
            does_not_target_member_type
                .into_iter()
                .map(
                    |inline_fragment| Error::InlineFragmentOnUnionDoesNotTargetMember {
                        inline_fragment,
                        union_type_definition,
                    },
                ),
        );

        self.errors.extend(
            bluejay_validator::utils::duplicates(
                targets_member_type.into_iter(),
                |inline_fragment: &E::InlineFragment| inline_fragment.type_condition().unwrap(),
            )
            .map(|(type_condition, inline_fragments)| {
                Error::NonUniqueInlineFragmentTypeConditions {
                    type_condition,
                    selection_set,
                    inline_fragments,
                }
            }),
        );
    }

    fn typename_selection(
        &self,
        selection_set: &'a E::SelectionSet,
    ) -> (Option<&'a E::Field>, Vec<&'a E::Field>) {
        let (typename, other): (Vec<_>, Vec<_>) = selection_set
            .iter()
            .filter_map(|selection| match selection.as_ref() {
                SelectionReference::Field(f) => Some(f),
                SelectionReference::InlineFragment(_) | SelectionReference::FragmentSpread(_) => {
                    None
                }
            })
            .partition(|f| f.alias().is_none() && f.name() == "__typename");

        (typename.first().copied(), other)
    }

    /// Produces error if a fragment spread is present and not the only selection.
    /// Returns a boolean indicating if a fragment spread is present.
    fn validate_fragment_spreads(&mut self, selection_set: &'a E::SelectionSet) -> bool {
        let fragment_spread = selection_set
            .iter()
            .find_map(|selection| match selection.as_ref() {
                SelectionReference::FragmentSpread(fs) => Some(fs),
                _ => None,
            });

        match fragment_spread {
            Some(fragment_spread) if selection_set.len() != 1 => {
                self.errors.push(Error::FragmentSpreadNotIsolated {
                    selection_set,
                    fragment_spread,
                });
            }
            _ => {}
        }

        fragment_spread.is_some()
    }
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Rule<'a, E, S>
    for SelectionsAreValid<'a, E, S>
{
    type Error = Error<'a, E, S>;
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> IntoIterator for SelectionsAreValid<'a, E, S> {
    type Item = Error<'a, E, S>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}
