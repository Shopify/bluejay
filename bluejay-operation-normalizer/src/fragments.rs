use bluejay_core::executable::{
    ExecutableDocument, Field, FragmentDefinition, FragmentSpread, InlineFragment, Selection,
    SelectionReference,
};
use bluejay_core::AsIter;
use std::collections::{HashMap, HashSet};

pub(crate) fn collect_used_fragments<'a, E: ExecutableDocument + 'a>(
    selection_set: &'a E::SelectionSet,
    fragment_defs: &HashMap<&'a str, &'a E::FragmentDefinition>,
) -> Vec<&'a str> {
    let mut seen = HashSet::new();
    let mut stack: Vec<&'a E::SelectionSet> = vec![selection_set];

    while let Some(ss) = stack.pop() {
        for selection in ss.iter() {
            match selection.as_ref() {
                SelectionReference::Field(field) => {
                    if let Some(sub_ss) = field.selection_set() {
                        stack.push(sub_ss);
                    }
                }
                SelectionReference::FragmentSpread(spread) => {
                    let name = spread.name();
                    if seen.insert(name) {
                        if let Some(frag_def) = fragment_defs.get(name) {
                            stack.push(frag_def.selection_set());
                        }
                    }
                }
                SelectionReference::InlineFragment(inline) => {
                    stack.push(inline.selection_set());
                }
            }
        }
    }

    let mut used: Vec<_> = seen.into_iter().collect();
    used.sort_unstable();
    used
}
