use std::cmp::{Eq, Ord};
use std::collections::BTreeMap;
use std::hash::Hash;

pub fn duplicates<T: Copy, I: Iterator<Item = T>, K: Hash + Ord + Eq>(
    iter: I,
    key: fn(T) -> K,
) -> impl Iterator<Item = (K, Vec<T>)> {
    let indexed = iter.fold(
        BTreeMap::new(),
        |mut indexed: BTreeMap<K, Vec<T>>, el: T| {
            indexed.entry(key(el)).or_default().push(el);
            indexed
        },
    );

    indexed.into_iter().filter(|(_, values)| values.len() > 1)
}
