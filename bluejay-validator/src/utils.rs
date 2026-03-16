use std::cmp::{Eq, Ord};
use std::collections::BTreeMap;
use std::hash::Hash;

pub fn duplicates<T: Copy, I: Iterator<Item = T>, K: Hash + Ord + Eq + Copy>(
    iter: I,
    key: fn(T) -> K,
) -> impl Iterator<Item = (K, Vec<T>)> {
    // Collect items first to check for duplicates without BTreeMap
    let items: Vec<T> = iter.collect();

    // If 0 or 1 items, no duplicates possible — avoid any allocation
    if items.len() <= 1 {
        return Vec::new().into_iter();
    }

    // Quick O(n²) check for duplicates before allocating BTreeMap
    let has_dupes = items.iter().enumerate().any(|(i, el)| {
        let k = key(*el);
        items[..i].iter().any(|prev| key(*prev) == k)
    });

    if !has_dupes {
        return Vec::new().into_iter();
    }

    // Only allocate BTreeMap when we know there are duplicates
    let mut indexed = BTreeMap::new();
    for el in items {
        indexed
            .entry(key(el))
            .or_insert_with(Vec::new)
            .push(el);
    }

    indexed
        .into_iter()
        .filter(|(_, values)| values.len() > 1)
        .collect::<Vec<_>>()
        .into_iter()
}
