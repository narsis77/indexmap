
use std::hash::Hash;
use std::hash::BuildHasher;

use super::{OrderMap, Equivalent};

/// Opt-in mutable access to keys.
///
/// You are allowed to modify the keys in the hashmap **if the modifcation
/// does not change the key's hash and equality**.
///
/// If keys are modified erronously, you can no longer look them up.
///
/// These methods expose `&mut K`, mutable references to the key as it is stored
/// in the map. The key is allowed to be modified, but *only in a way that
/// preserves its hash and equality* (it is only useful for composite key
/// structs).
///
/// This is sound (memory safe) but a logical error hazard (just like
/// implementing PartialEq, Eq, or Hash incorrectly would be).
///
pub trait MutableKeys {
    type Key;
    type Value;
    
    /// Return item index, mutable reference to key and value
    fn get_full_mut2<Q: ?Sized>(&mut self, key: &Q)
        -> Option<(usize, &mut Self::Key, &mut Self::Value)>
        where Q: Hash + Equivalent<Self::Key>;

    /// Scan through each key-value pair in the map and keep those where the
    /// closure `keep` returns `true`.
    ///
    /// The order the elements are visited is not specified.
    ///
    /// Computes in **O(n)** time (average).
    fn retain2<F>(&mut self, keep: F)
        where F: FnMut(&mut Self::Key, &mut Self::Value) -> bool;
}

impl<K, V, S> MutableKeys for OrderMap<K, V, S>
    where K: Eq + Hash,
          S: BuildHasher,
{
    type Key = K;
    type Value = V;
    fn get_full_mut2<Q: ?Sized>(&mut self, key: &Q)
        -> Option<(usize, &mut K, &mut V)>
        where Q: Hash + Equivalent<K>,
    {
        if let Some((_, found)) = self.find(key) {
            let entry = &mut self.entries[found];
            Some((found, &mut entry.key, &mut entry.value))
        } else {
            None
        }
    }

    fn retain2<F>(&mut self, mut keep: F)
        where F: FnMut(&mut K, &mut V) -> bool,
    {
        // We can use either forward or reverse scan, but forward was
        // faster in a microbenchmark
        let mut i = 0;
        while i < self.len() {
            {
                let entry = &mut self.entries[i];
                if keep(&mut entry.key, &mut entry.value) {
                    i += 1;
                    continue;
                }
            }
            self.swap_remove_index(i);
            // skip increment on remove
        }
    }
}
