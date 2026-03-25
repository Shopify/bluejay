use bluejay_core::executable::{
    ExecutableDocument, Field, FragmentDefinition, FragmentSpread, InlineFragment, Selection,
    SelectionReference,
};
use bluejay_core::{Argument, AsIter, Directive};
use std::collections::{HashMap, HashSet};

use crate::ir::{
    NormalizedDirective, NormalizedField, NormalizedInlineFragment, NormalizedSelection,
};

pub(crate) fn build_selections<'a, E: ExecutableDocument + 'a>(
    selection_set: &'a E::SelectionSet,
    fragment_defs: &HashMap<&'a str, &'a E::FragmentDefinition>,
    expanding: &mut HashSet<&'a str>,
) -> Vec<NormalizedSelection<'a>> {
    let mut result = Vec::with_capacity(selection_set.len());

    for selection in selection_set.iter() {
        match selection.as_ref() {
            SelectionReference::Field(field) => {
                result.push(NormalizedSelection::Field(build_field::<E>(
                    field,
                    fragment_defs,
                    expanding,
                )));
            }
            SelectionReference::FragmentSpread(spread) => {
                let name = spread.name();
                if expanding.contains(name) {
                    continue; // cycle — skip
                }
                if let Some(frag_def) = fragment_defs.get(name) {
                    expanding.insert(name);

                    // Merge spread directives + fragment definition directives
                    let mut directives =
                        build_directives::<false, E>(spread.directives());
                    directives.extend(build_directives::<false, E>(
                        frag_def.directives(),
                    ));
                    directives.sort_unstable();

                    let selections = build_selections::<E>(
                        frag_def.selection_set(),
                        fragment_defs,
                        expanding,
                    );

                    expanding.remove(name);

                    result.push(NormalizedSelection::InlineFragment(
                        NormalizedInlineFragment {
                            type_condition: Some(frag_def.type_condition()),
                            directives,
                            selections,
                        },
                    ));
                }
            }
            SelectionReference::InlineFragment(inline) => {
                let directives =
                    build_directives::<false, E>(inline.directives());
                let selections =
                    build_selections::<E>(inline.selection_set(), fragment_defs, expanding);

                result.push(NormalizedSelection::InlineFragment(
                    NormalizedInlineFragment {
                        type_condition: inline.type_condition(),
                        directives,
                        selections,
                    },
                ));
            }
        }
    }

    result
}

fn build_field<'a, E: ExecutableDocument + 'a>(
    field: &'a E::Field,
    fragment_defs: &HashMap<&'a str, &'a E::FragmentDefinition>,
    expanding: &mut HashSet<&'a str>,
) -> NormalizedField<'a> {
    let arg_names = build_arg_names::<false, E>(field.arguments());
    let directives = build_directives::<false, E>(field.directives());
    let selections = match field.selection_set() {
        Some(ss) => build_selections::<E>(ss, fragment_defs, expanding),
        None => Vec::new(),
    };

    NormalizedField {
        name: field.name(), // alias dropped
        arg_names,
        directives,
        selections,
    }
}

fn build_arg_names<'a, const CONST: bool, E: ExecutableDocument + 'a>(
    args: Option<&'a E::Arguments<CONST>>,
) -> Vec<&'a str> {
    let Some(args) = args else {
        return Vec::new();
    };
    let mut names: Vec<&str> = args.iter().map(|a| a.name()).collect();
    names.sort_unstable();
    names
}

pub(crate) fn build_directives<'a, const CONST: bool, E: ExecutableDocument + 'a>(
    directives: Option<&'a E::Directives<CONST>>,
) -> Vec<NormalizedDirective<'a>> {
    let Some(directives) = directives else {
        return Vec::new();
    };
    let mut result: Vec<_> = directives
        .iter()
        .map(|d| NormalizedDirective {
            name: d.name(),
            arg_names: {
                let mut names: Vec<&str> = match d.arguments() {
                    Some(args) => args.iter().map(|a| a.name()).collect(),
                    None => Vec::new(),
                };
                names.sort_unstable();
                names
            },
        })
        .collect();
    result.sort_unstable();
    result
}
