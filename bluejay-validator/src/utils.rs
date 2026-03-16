use std::cmp::{Eq, Ord};
use std::collections::BTreeMap;
use std::hash::Hash;

pub fn duplicates<T: Copy, I: Iterator<Item = T>, K: Hash + Ord + Eq>(
    iter: I,
    key: fn(T) -> K,
) -> impl Iterator<Item = (K, Vec<T>)> {
    // Fast path: collect into a small vec and check for duplicates with linear scan
    // This avoids BTreeMap allocation for the common case (no duplicates, few items)
    let items: Vec<(K, T)> = iter.map(|el| (key(el), el)).collect();

    // If 0 or 1 items, no duplicates possible
    if items.len() <= 1 {
        return Vec::new().into_iter();
    }

    // Check if any duplicates exist before allocating the BTreeMap
    let has_duplicates = items.iter().enumerate().any(|(i, (k1, _))| {
        items[i + 1..].iter().any(|(k2, _)| k1 == k2)
    });

    if !has_duplicates {
        return Vec::new().into_iter();
    }

    // Only allocate BTreeMap when we know there are duplicates
    let mut indexed = BTreeMap::new();
    for (k, el) in items {
        indexed.entry(k).or_insert_with(Vec::new).push(el);
    }

    indexed
        .into_iter()
        .filter(|(_, values)| values.len() > 1)
        .collect::<Vec<_>>()
        .into_iter()
}
