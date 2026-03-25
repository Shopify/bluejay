use std::cmp::Ordering;

use crate::ir::NormalizedSelection;

/// Normalize a selection vector in-place: flatten, merge, sort, recurse.
pub(crate) fn normalize_selections(selections: &mut Vec<NormalizedSelection<'_>>) {
    let has_inline_fragments = selections
        .iter()
        .any(|s| matches!(s, NormalizedSelection::InlineFragment(_)));

    if has_inline_fragments {
        flatten_bare_inline_fragments(selections);
        merge_inline_fragments(selections);
    }

    sort_selections(selections);

    for sel in selections.iter_mut() {
        match sel {
            NormalizedSelection::Field(f) => {
                if !f.selections.is_empty() {
                    normalize_selections(&mut f.selections);
                }
            }
            NormalizedSelection::InlineFragment(inf) => {
                normalize_selections(&mut inf.selections);
            }
        }
    }
}

/// Flatten InlineFragments that have no type condition and no directives.
/// Their children get spliced into the parent. Handles nested bare IFs.
fn flatten_bare_inline_fragments(selections: &mut Vec<NormalizedSelection<'_>>) {
    // Check if any bare IFs exist before allocating
    let has_bare = selections.iter().any(|s| {
        matches!(s, NormalizedSelection::InlineFragment(inf)
            if inf.type_condition.is_none() && inf.directives.is_empty())
    });
    if !has_bare {
        return;
    }

    let mut result = Vec::with_capacity(selections.len());
    let mut stack: Vec<NormalizedSelection<'_>> =
        std::mem::take(selections).into_iter().rev().collect();

    while let Some(sel) = stack.pop() {
        match sel {
            NormalizedSelection::InlineFragment(inf)
                if inf.type_condition.is_none() && inf.directives.is_empty() =>
            {
                for child in inf.selections.into_iter().rev() {
                    stack.push(child);
                }
            }
            other => result.push(other),
        }
    }

    *selections = result;
}

/// Merge InlineFragments with matching (type_condition, directives).
fn merge_inline_fragments(selections: &mut Vec<NormalizedSelection<'_>>) {
    // Count IFs — if 0 or 1, nothing to merge
    let if_count = selections
        .iter()
        .filter(|s| matches!(s, NormalizedSelection::InlineFragment(_)))
        .count();
    if if_count < 2 {
        return;
    }

    let mut merged: Vec<NormalizedSelection<'_>> = Vec::with_capacity(selections.len());

    for sel in std::mem::take(selections) {
        match sel {
            NormalizedSelection::Field(_) => merged.push(sel),
            NormalizedSelection::InlineFragment(inf) => {
                let existing = merged.iter().position(|existing| {
                    matches!(existing, NormalizedSelection::InlineFragment(existing_inf)
                        if existing_inf.type_condition == inf.type_condition
                        && existing_inf.directives == inf.directives)
                });

                if let Some(pos) = existing {
                    if let NormalizedSelection::InlineFragment(ref mut target) = merged[pos] {
                        target.selections.extend(inf.selections);
                    }
                } else {
                    merged.push(NormalizedSelection::InlineFragment(inf));
                }
            }
        }
    }

    *selections = merged;
}

fn sort_selections(selections: &mut [NormalizedSelection<'_>]) {
    selections.sort_unstable_by(|a, b| match (a, b) {
        (NormalizedSelection::Field(af), NormalizedSelection::Field(bf)) => {
            af.name.cmp(bf.name)
        }
        (NormalizedSelection::Field(_), NormalizedSelection::InlineFragment(_)) => Ordering::Less,
        (NormalizedSelection::InlineFragment(_), NormalizedSelection::Field(_)) => {
            Ordering::Greater
        }
        (
            NormalizedSelection::InlineFragment(ai),
            NormalizedSelection::InlineFragment(bi),
        ) => ai
            .type_condition
            .cmp(&bi.type_condition)
            .then_with(|| ai.directives.cmp(&bi.directives)),
    });
}
