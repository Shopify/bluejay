use crate::validation::Error;
use bluejay_core::{
    definition::{
        InterfaceImplementation, InterfaceTypeDefinition, SchemaDefinition,
        TypeDefinitionReference, UnionMemberTypes, UnionTypeDefinition,
    },
    executable::{
        ExecutableDocument, Field, FragmentDefinition, FragmentSpread, InlineFragment, Selection,
        SelectionReference,
    },
    AsIter,
};
use bluejay_validator::executable::{
    document::{Rule, Visitor},
    Cache,
};
use itertools::{Either, Itertools};

pub(crate) struct SelectionsAreValid<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> {
    errors: Vec<Error<'a, E, S>>,
    cache: &'a Cache<'a, E, S>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Visitor<'a, E, S>
    for SelectionsAreValid<'a, E, S>
{
    fn new(_: &'a E, _: &'a S, cache: &'a Cache<'a, E, S>) -> Self {
        Self {
            errors: Vec::new(),
            cache,
        }
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
            TypeDefinitionReference::Interface(itd) => {
                self.visit_interface_selection_set(selection_set, itd)
            }
            _ => {}
        }
    }
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> SelectionsAreValid<'a, E, S> {
    fn visit_object_selection_set(&mut self, selection_set: &'a E::SelectionSet) {
        match self.isolated_fragment_spread(selection_set) {
            Ok(_) => {} // any fragment spread on an object type that is valid by the builtin rules will always be present so is valid
            Err(e) => self.errors.push(e),
        }

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

    fn visit_interface_selection_set(
        &mut self,
        selection_set: &'a E::SelectionSet,
        interface_type_definition: &'a S::InterfaceTypeDefinition,
    ) {
        match self.isolated_fragment_spread(selection_set) {
            Ok(Some(fragment_spread)) => {
                // Currently, the generated type assumes that the fragment spread is always present.
                // Therefore, we need to ensure that the target type of the fragment spread is either
                // the interface, or one of the interfaces that it implements. Otherwise, there is a chance
                // that the generated type will not be able to deserialize the fragment spread because the
                // fields are not present. For example, if the fragment spread targets one of several types
                // that implement the interface:
                // Schema:
                // ```graphql
                // interface Node { id: ID! }
                // type User implements Node { id: ID!, name: String! }
                // type Post implements Node { id: ID!, title: String! }
                // type Query { node: Node }
                // ```
                // Query:
                // ```graphql
                // query {
                //   node {
                //     ...UserFragment
                //   }
                // }
                // fragment UserFragment on User {
                //   id
                //   name
                // }
                // ```
                // A perfectly valid response would be the `Post` case, but we'd be unable to deserialize it:
                // ```json
                // {
                //   "node": {}
                // }
                // ```
                let fragment_definition = self
                    .cache
                    .fragment_definition(fragment_spread.name())
                    .unwrap();
                let type_condition = fragment_definition.type_condition();
                let mut allowed_types_iter = interface_type_definition
                    .interface_implementations()
                    .map(|interface_implementations| {
                        interface_implementations
                            .iter()
                            .map(InterfaceImplementation::name)
                    })
                    .into_iter()
                    .flatten()
                    .chain(std::iter::once(interface_type_definition.name()));
                if !allowed_types_iter.any(|allowed_type| allowed_type == type_condition) {
                    self.errors
                        .push(Error::FragmentSpreadOnInterfaceInvalidTarget { fragment_spread });
                }
            }
            Ok(None) => {}
            Err(e) => self.errors.push(e),
        }

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
        match self.isolated_fragment_spread(selection_set) {
            Ok(Some(_)) => {
                return;
            }
            Ok(None) => {}
            Err(e) => self.errors.push(e),
        }

        let (typename_first_selection, other_field_selections) =
            self.typename_first_selection(selection_set);

        if typename_first_selection.is_none() {
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

    /// Returns a 2-tuple
    /// - first element: an option including a field if that field is the first selection in the set and is named `__typename` without an alias
    /// - second element: a vector of all other fields in the selection set
    fn typename_first_selection(
        &self,
        selection_set: &'a E::SelectionSet,
    ) -> (Option<&'a E::Field>, Vec<&'a E::Field>) {
        let (typename, other): (Vec<_>, Vec<_>) = selection_set
            .iter()
            .enumerate()
            .filter_map(|(idx, selection)| match selection.as_ref() {
                SelectionReference::Field(f) => Some((idx, f)),
                SelectionReference::InlineFragment(_) | SelectionReference::FragmentSpread(_) => {
                    None
                }
            })
            .partition_map(|(idx, f)| {
                if idx == 0 && f.alias().is_none() && f.name() == "__typename" {
                    Either::Left(f)
                } else {
                    Either::Right(f)
                }
            });

        (typename.first().copied(), other)
    }

    /// Returns an error if the selection set contains a fragment spread that is not the only selection is the set.
    /// Otherwise returns an option of the fragment spread, if present.
    fn isolated_fragment_spread(
        &self,
        selection_set: &'a E::SelectionSet,
    ) -> Result<Option<&'a E::FragmentSpread>, Error<'a, E, S>> {
        let fragment_spread = selection_set
            .iter()
            .find_map(|selection| match selection.as_ref() {
                SelectionReference::FragmentSpread(fs) => Some(fs),
                _ => None,
            });

        match fragment_spread {
            Some(fragment_spread) if selection_set.len() != 1 => {
                Err(Error::FragmentSpreadNotIsolated {
                    selection_set,
                    fragment_spread,
                })
            }
            fs => Ok(fs),
        }
    }
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Rule<'a, E, S>
    for SelectionsAreValid<'a, E, S>
{
    type Error = Error<'a, E, S>;
    type Errors = std::vec::IntoIter<Self::Error>;

    fn into_errors(self) -> Self::Errors {
        self.errors.into_iter()
    }
}
