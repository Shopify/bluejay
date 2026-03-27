use itertools::Itertools;
use std::cmp::{Eq, Ord};
use std::collections::BTreeMap;
use std::hash::Hash;

pub fn duplicates<T: Copy, I: Iterator<Item = T>, K: Hash + Ord + Eq + Copy>(
    mut iter: I,
    key: fn(T) -> K,
) -> impl Iterator<Item = (K, Vec<T>)> {
    // If 0 or 1 items, no duplicates possible — avoid any allocation
    let Some((first, second)) = iter.next().zip(iter.next()) else {
        return Vec::new().into_iter();
    };

    let items: Vec<T> = [first, second].into_iter().chain(iter).collect();

    // Quick O(n²) check for duplicates before allocating BTreeMap
    let has_dupes = items
        .iter()
        .array_combinations()
        .any(|[a, b]| key(*a) == key(*b));

    if !has_dupes {
        return Vec::new().into_iter();
    }

    // Only allocate BTreeMap when we know there are duplicates
    let mut indexed = BTreeMap::new();
    for el in items {
        indexed.entry(key(el)).or_insert_with(Vec::new).push(el);
    }

    indexed
        .into_iter()
        .filter(|(_, values)| values.len() > 1)
        .collect::<Vec<_>>()
        .into_iter()
}
