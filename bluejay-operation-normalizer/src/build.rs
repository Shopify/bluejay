//! Builds the normalized IR from a parsed AST in a single recursive pass.
//!
//! Implements algorithm steps 2a–2e: for each selection set, this module builds
//! normalized fields and inline fragments, expands fragment spreads, flattens
//! bare inline fragments, merges matching inline fragments, and sorts — all
//! bottom-up so each level is fully normalized before being returned to the parent.

use bluejay_core::executable::{
    ExecutableDocument, Field, FragmentDefinition, FragmentSpread, InlineFragment, Selection,
    SelectionReference,
};
use bluejay_core::{Argument, AsIter, Directive};
use bumpalo::collections::Vec as BVec;
use bumpalo::Bump;
use std::cmp::Ordering;

use crate::ir::{
    NormalizedDirective, NormalizedField, NormalizedInlineFragment, NormalizedSelection,
};

/// Build and normalize a selection set in a single recursive pass (steps 2a–2e).
///
/// For each selection in the set:
/// - **Fields** (step 2a): collect name (alias dropped), sorted args, sorted directives,
///   and recursively build child selections.
/// - **Fragment spreads** (step 2b): expand to inline fragments with the fragment's type
///   condition, merging directives from both spread and definition.
/// - **Inline fragments** (step 2c): flatten bare ones (no type condition, no directives)
///   into the parent; keep others as-is.
///
/// After collecting all selections, merge and sort (steps 2d–2e) via [`normalize_in_place`].
pub(crate) fn build_selections<'a, 'bump, E: ExecutableDocument + 'a>(
    selection_set: &'a E::SelectionSet,
    fragment_defs: &[(&'a str, &'a E::FragmentDefinition)],
    expanding: &mut Vec<&'a str>,
    bump: &'bump Bump,
) -> BVec<'bump, NormalizedSelection<'a, 'bump>> {
    let mut result = BVec::with_capacity_in(selection_set.len(), bump);

    for selection in selection_set.iter() {
        match selection.as_ref() {
            // Step 2a: fields
            SelectionReference::Field(field) => {
                result.push(NormalizedSelection::Field(build_field::<E>(
                    field,
                    fragment_defs,
                    expanding,
                    bump,
                )));
            }
            // Step 2b: expand fragment spreads into inline fragments
            SelectionReference::FragmentSpread(spread) => {
                let name = spread.name();
                // Cycle detection: skip if this fragment is already being expanded
                if expanding.contains(&name) {
                    continue;
                }
                if let Some((_, frag_def)) = fragment_defs.iter().find(|(n, _)| *n == name) {
                    expanding.push(name);

                    let mut directives = build_directives::<false, E>(spread.directives(), bump);
                    directives.extend(build_directives::<false, E>(frag_def.directives(), bump));
                    directives.sort_unstable();

                    let selections = build_selections::<E>(
                        frag_def.selection_set(),
                        fragment_defs,
                        expanding,
                        bump,
                    );

                    expanding.pop();

                    result.push(NormalizedSelection::InlineFragment(
                        NormalizedInlineFragment {
                            type_condition: Some(frag_def.type_condition()),
                            directives,
                            selections,
                        },
                    ));
                }
            }
            // Step 2c: inline fragments — flatten bare ones, keep others
            SelectionReference::InlineFragment(inline) => {
                let directives = build_directives::<false, E>(inline.directives(), bump);
                let selections =
                    build_selections::<E>(inline.selection_set(), fragment_defs, expanding, bump);

                // Flatten bare inline fragments (no type condition, no directives)
                if inline.type_condition().is_none() && directives.is_empty() {
                    result.extend(selections);
                } else {
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
    }

    // Merge inline fragments with same (type_condition, directives) and sort
    normalize_in_place(&mut result);

    result
}

/// Steps 2d–2e: merge inline fragments with matching `(type_condition, directives)`,
/// then sort all selections (fields first by name, then inline fragments by type
/// condition and directives).
fn normalize_in_place(selections: &mut BVec<'_, NormalizedSelection<'_, '_>>) {
    let mut if_count = 0u32;
    for s in selections.iter() {
        if matches!(s, NormalizedSelection::InlineFragment(_)) {
            if_count += 1;
            if if_count >= 2 {
                break;
            }
        }
    }

    if if_count >= 2 {
        let mut i = 0;
        while i < selections.len() {
            if let NormalizedSelection::InlineFragment(_) = &selections[i] {
                let mut j = i + 1;
                let mut merged = false;
                while j < selections.len() {
                    let should_merge = match (&selections[i], &selections[j]) {
                        (
                            NormalizedSelection::InlineFragment(a),
                            NormalizedSelection::InlineFragment(b),
                        ) => a.type_condition == b.type_condition && a.directives == b.directives,
                        _ => false,
                    };
                    if should_merge {
                        let removed = selections.swap_remove(j);
                        if let NormalizedSelection::InlineFragment(inf) = removed {
                            if let NormalizedSelection::InlineFragment(ref mut target) =
                                selections[i]
                            {
                                target.selections.extend(inf.selections);
                                merged = true;
                            }
                        }
                    } else {
                        j += 1;
                    }
                }
                // Only re-sort if we actually merged something
                if merged {
                    if let NormalizedSelection::InlineFragment(ref mut target) = selections[i] {
                        target
                            .selections
                            .sort_unstable_by(|a, b| cmp_selections(a, b));
                    }
                }
            }
            i += 1;
        }
    }

    selections.sort_unstable_by(|a, b| cmp_selections(a, b));
}

/// Sort order for step 2e: fields first (alphabetically by name), then inline
/// fragments (by type condition, then by directives).
fn cmp_selections(a: &NormalizedSelection<'_, '_>, b: &NormalizedSelection<'_, '_>) -> Ordering {
    match (a, b) {
        (NormalizedSelection::Field(af), NormalizedSelection::Field(bf)) => af
            .name
            .cmp(bf.name)
            .then_with(|| af.arg_names.as_slice().cmp(bf.arg_names.as_slice()))
            .then_with(|| af.directives.cmp(&bf.directives)),
        (NormalizedSelection::Field(_), NormalizedSelection::InlineFragment(_)) => Ordering::Less,
        (NormalizedSelection::InlineFragment(_), NormalizedSelection::Field(_)) => {
            Ordering::Greater
        }
        (NormalizedSelection::InlineFragment(ai), NormalizedSelection::InlineFragment(bi)) => ai
            .type_condition
            .cmp(&bi.type_condition)
            .then_with(|| ai.directives.cmp(&bi.directives)),
    }
}

/// Step 2a: build a normalized field — alias dropped, args/directives sorted,
/// child selections recursively normalized.
fn build_field<'a, 'bump, E: ExecutableDocument + 'a>(
    field: &'a E::Field,
    fragment_defs: &[(&'a str, &'a E::FragmentDefinition)],
    expanding: &mut Vec<&'a str>,
    bump: &'bump Bump,
) -> NormalizedField<'a, 'bump> {
    let arg_names = build_arg_names::<false, E>(field.arguments(), bump);
    let directives = build_directives::<false, E>(field.directives(), bump);
    let selections = match field.selection_set() {
        Some(ss) => build_selections::<E>(ss, fragment_defs, expanding, bump),
        None => BVec::new_in(bump),
    };

    NormalizedField {
        name: field.name(),
        arg_names,
        directives,
        selections,
    }
}

/// Collect and sort argument names alphabetically. Values are erased during
/// serialization (step 3) — only names matter for the canonical form.
fn build_arg_names<'a, 'bump, const CONST: bool, E: ExecutableDocument + 'a>(
    args: Option<&'a E::Arguments<CONST>>,
    bump: &'bump Bump,
) -> BVec<'bump, &'a str> {
    let Some(args) = args else {
        return BVec::new_in(bump);
    };
    let mut names: BVec<'bump, &'a str> = BVec::from_iter_in(args.iter().map(|a| a.name()), bump);
    names.sort_unstable();
    names
}

/// Collect and sort directives by name (then by argument names). Each directive's
/// argument names are also sorted. Used for fields, inline fragments, and operations.
pub(crate) fn build_directives<'a, 'bump, const CONST: bool, E: ExecutableDocument + 'a>(
    directives: Option<&'a E::Directives<CONST>>,
    bump: &'bump Bump,
) -> BVec<'bump, NormalizedDirective<'a, 'bump>> {
    let Some(directives) = directives else {
        return BVec::new_in(bump);
    };
    let mut result: BVec<'bump, _> = BVec::from_iter_in(
        directives.iter().map(|d| NormalizedDirective {
            name: d.name(),
            arg_names: {
                let mut names: BVec<'bump, &str> = match d.arguments() {
                    Some(args) => BVec::from_iter_in(args.iter().map(|a| a.name()), bump),
                    None => BVec::new_in(bump),
                };
                names.sort_unstable();
                names
            },
        }),
        bump,
    );
    result.sort_unstable();
    result
}
