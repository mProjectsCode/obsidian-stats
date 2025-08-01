use hashbrown::HashMap;
use std::hash::Hash;

pub fn group_by<T, K: Hash + Eq, TFn: Fn(&T) -> K>(
    items: impl IntoIterator<Item = T>,
    key_fn: TFn,
) -> HashMap<K, Vec<T>> {
    let mut map: HashMap<K, Vec<T>> = HashMap::new();
    for item in items {
        let key = key_fn(&item);
        map.entry(key).or_default().push(item);
    }
    map
}

pub trait GroupByExt<T, K: Hash + Eq> {
    fn group_by<F: Fn(&T) -> K>(self, key_fn: F) -> impl Iterator<Item = (K, Vec<T>)>;
}

impl<T, K: Hash + Eq, I: IntoIterator<Item = T>> GroupByExt<T, K> for I {
    fn group_by<F: Fn(&T) -> K>(self, key_fn: F) -> impl Iterator<Item = (K, Vec<T>)> {
        group_by(self, key_fn).into_iter()
    }
}

pub trait SortExt<T> {
    /// Sort the items in the iterator using a custom comparison function.
    /// This collects the items into a vector, sorts them, and returns an iterator over the sorted items.
    fn sort_by<F: Fn(&T, &T) -> std::cmp::Ordering>(self, compare_fn: F)
    -> impl Iterator<Item = T>;
}

impl<T, I: Iterator<Item = T>> SortExt<T> for I {
    fn sort_by<F: Fn(&T, &T) -> std::cmp::Ordering>(
        self,
        compare_fn: F,
    ) -> impl Iterator<Item = T> {
        let mut items: Vec<T> = self.collect();
        items.sort_by(compare_fn);
        items.into_iter()
    }
}

pub trait DedupExt<T> {
    /// Deduplicate the items in the iterator based on a custom equality function.
    /// This collects the items into a vector, deduplicates them, and returns an iterator over the unique items.
    /// This is O(n^2) in the worst case, but I don't care.
    fn dedup_by<F: Fn(&T, &T) -> bool>(self, eq_fn: F) -> impl Iterator<Item = T>;
}

impl<T, I: Iterator<Item = T>> DedupExt<T> for I {
    fn dedup_by<F: Fn(&T, &T) -> bool>(self, eq_fn: F) -> impl Iterator<Item = T> {
        let mut items: Vec<T> = vec![];
        for item in self {
            if items.iter().all(|existing| !eq_fn(existing, &item)) {
                items.push(item);
            }
        }
        items.into_iter()
    }
}
