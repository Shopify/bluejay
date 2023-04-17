use crate::executable::{Cache, Error, Rule, Visitor};
use bluejay_core::definition::SchemaDefinition;
use bluejay_core::executable::{
    ExecutableDocument, Field, FragmentDefinition, FragmentSpread, InlineFragment, Selection,
};
use std::collections::{BTreeMap, HashSet};

pub struct FragmentSpreadsMustNotFormCycles<'a, E: ExecutableDocument, S: SchemaDefinition> {
    errors: Vec<Error<'a, E, S>>,
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> FragmentSpreadsMustNotFormCycles<'a, E, S> {
    fn detect_fragment_cycles(
        spreads_by_fragment_definition: &BTreeMap<
            &'a str,
            (&'a E::FragmentDefinition, Vec<&'a E::FragmentSpread>),
        >,
        target: &'a E::FragmentDefinition,
        fragment_definition_name: &'a str,
        visited_fragment_definitions: &HashSet<&'a str>,
    ) -> Option<Error<'a, E, S>> {
        if visited_fragment_definitions.contains(fragment_definition_name) {
            return None;
        }
        if let Some((_, spreads)) = spreads_by_fragment_definition.get(fragment_definition_name) {
            let mut visited_fragment_definitions = visited_fragment_definitions.clone();
            visited_fragment_definitions.insert(fragment_definition_name);
            spreads.iter().find_map(|&spread| {
                if spread.name() == target.name() {
                    Some(Error::FragmentSpreadCycle {
                        fragment_definition: target,
                        fragment_spread: spread,
                    })
                } else {
                    Self::detect_fragment_cycles(
                        spreads_by_fragment_definition,
                        target,
                        spread.name(),
                        &visited_fragment_definitions,
                    )
                }
            })
        } else {
            None
        }
    }
}

impl<'a, E: ExecutableDocument, S: SchemaDefinition> Visitor<'a, E, S>
    for FragmentSpreadsMustNotFormCycles<'a, E, S>
{
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> IntoIterator
    for FragmentSpreadsMustNotFormCycles<'a, E, S>
{
    type Item = Error<'a, E, S>;
    type IntoIter = std::vec::IntoIter<Error<'a, E, S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<'a, E: ExecutableDocument + 'a, S: SchemaDefinition + 'a> Rule<'a, E, S>
    for FragmentSpreadsMustNotFormCycles<'a, E, S>
{
    fn new(executable_document: &'a E, _: &'a S, _: &'a Cache<'a, E, S>) -> Self {
        let spreads_by_fragment_definition: BTreeMap<
            &'a str,
            (&'a E::FragmentDefinition, Vec<&'a E::FragmentSpread>),
        > = BTreeMap::from_iter(executable_document.fragment_definitions().iter().map(
            |fragment_definition| {
                (
                    fragment_definition.name(),
                    (
                        fragment_definition,
                        contained_fragment_spreads::<E>(fragment_definition),
                    ),
                )
            },
        ));
        let errors = spreads_by_fragment_definition
            .iter()
            .filter_map(|(fragment_definition_name, (target, _))| {
                Self::detect_fragment_cycles(
                    &spreads_by_fragment_definition,
                    target,
                    fragment_definition_name,
                    &HashSet::new(),
                )
            })
            .collect();
        Self { errors }
    }
}

fn contained_fragment_spreads<'a, E: ExecutableDocument + 'a>(
    fragment_definition: &'a E::FragmentDefinition,
) -> Vec<&'a E::FragmentSpread> {
    let mut fragment_spreads = Vec::new();
    visit_selection_for_fragment_spreads::<E>(
        fragment_definition.selection_set(),
        &mut fragment_spreads,
    );
    fragment_spreads
}

fn visit_selection_for_fragment_spreads<'a, E: ExecutableDocument + 'a>(
    selection_set: &'a E::SelectionSet,
    fragment_spreads: &mut Vec<&'a E::FragmentSpread>,
) {
    selection_set
        .as_ref()
        .iter()
        .for_each(|selection| match selection.as_ref() {
            Selection::Field(f) => {
                if let Some(selection_set) = f.selection_set() {
                    visit_selection_for_fragment_spreads::<E>(selection_set, fragment_spreads)
                }
            }
            Selection::FragmentSpread(fs) => {
                fragment_spreads.push(fs);
            }
            Selection::InlineFragment(f) => {
                visit_selection_for_fragment_spreads::<E>(f.selection_set(), fragment_spreads)
            }
        })
}
