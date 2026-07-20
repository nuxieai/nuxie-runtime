//! Node: `cxx:Record:Luau.Common:Common/include/Luau/InsertionOrderedMap.h:15:insertion_ordered_map`
//! Source: `Common/include/Luau/InsertionOrderedMap.h:14-145` (hand-ported, complete API)
//!
//! A map preserving insertion order: pairs live in a Vec, an unordered index
//! maps key -> position. `insert` is first-write-wins (a duplicate key is a
//! no-op, matching C++). `erase` removes the pair and decrements every index
//! after it, exactly like the C++ loop. C++ `operator[]` (find-or-default-
//! insert) is `get_or_default`; C++ `find` returning an iterator maps to
//! `find` returning the position, with `get`/`get_mut` for the common
//! deref-the-iterator pattern.

extern crate alloc;

use crate::type_aliases::vec::vec;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct InsertionOrderedMap<K, V>
where
    K: Eq + Hash + Clone,
{
    pub(crate) pairs: vec<K, V>,
    pub(crate) indices: HashMap<K, usize>,
}

impl<K, V> InsertionOrderedMap<K, V>
where
    K: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self {
            pairs: vec::new(),
            indices: HashMap::new(),
        }
    }

    pub fn insert(&mut self, k: K, v: V) {
        if self.indices.contains_key(&k) {
            return;
        }
        self.pairs.push((k.clone(), v));
        self.indices.insert(k, self.pairs.len() - 1);
    }

    pub fn clear(&mut self) {
        self.pairs.clear();
        self.indices.clear();
    }

    pub fn size(&self) -> usize {
        debug_assert_eq!(self.pairs.len(), self.indices.len());
        self.pairs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    pub fn contains(&self, k: &K) -> bool {
        self.indices.contains_key(k)
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        self.indices.get(k).map(|&i| &self.pairs[i].1)
    }

    pub fn get_mut(&mut self, k: &K) -> Option<&mut V> {
        match self.indices.get(k) {
            Some(&i) => Some(&mut self.pairs[i].1),
            None => None,
        }
    }

    /// C++ `operator[]`: returns the value for `k`, default-inserting it at
    /// the back if absent.
    pub fn get_or_default(&mut self, k: K) -> &mut V
    where
        V: Default,
    {
        if let Some(&i) = self.indices.get(&k) {
            return &mut self.pairs[i].1;
        }
        self.pairs.push((k.clone(), V::default()));
        self.indices.insert(k, self.pairs.len() - 1);
        &mut self.pairs.last_mut().unwrap().1
    }

    /// C++ `find`: position of `k` in insertion order, or `None` (`end()`).
    pub fn find(&self, k: &K) -> Option<usize> {
        self.indices.get(k).copied()
    }

    /// C++ `erase(find(k))`: removes the pair and shifts every later index
    /// down by one. Absent keys are a no-op (erasing `end()`).
    pub fn erase(&mut self, k: &K) {
        let Some(removed) = self.indices.remove(k) else {
            return;
        };
        self.pairs.remove(removed);
        for index in self.indices.values_mut() {
            if *index > removed {
                *index -= 1;
            }
        }
    }

    pub fn iter(&self) -> core::slice::Iter<'_, (K, V)> {
        self.pairs.iter()
    }

    /// Values are mutable through iteration; keys are not (mutating a key
    /// would desynchronize `indices` — C++ merely trusts you not to).
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.pairs.iter_mut().map(|(k, v)| (&*k, v))
    }
}

impl<'a, K, V> IntoIterator for &'a InsertionOrderedMap<K, V>
where
    K: Eq + Hash + Clone,
{
    type Item = &'a (K, V);
    type IntoIter = core::slice::Iter<'a, (K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.pairs.iter()
    }
}

impl<K, V> Default for InsertionOrderedMap<K, V>
where
    K: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::InsertionOrderedMap;

    // Behavioral oracle: mirrors the C++ semantics in
    // Common/include/Luau/InsertionOrderedMap.h.
    #[test]
    fn insertion_order_preserved_and_duplicate_insert_is_noop() {
        let mut m: InsertionOrderedMap<i32, &str> = InsertionOrderedMap::new();
        m.insert(3, "c");
        m.insert(1, "a");
        m.insert(2, "b");
        m.insert(1, "OVERWRITE"); // C++: duplicate key is a no-op
        let keys: Vec<i32> = m.iter().map(|(k, _)| *k).collect();
        assert_eq!(keys, vec![3, 1, 2]);
        assert_eq!(m.get(&1), Some(&"a"));
        assert_eq!(m.size(), 3);
    }

    #[test]
    fn erase_reindexes_later_entries() {
        let mut m: InsertionOrderedMap<i32, i32> = InsertionOrderedMap::new();
        for k in [10, 20, 30, 40] {
            m.insert(k, k * 2);
        }
        m.erase(&20);
        assert_eq!(m.size(), 3);
        assert_eq!(m.find(&10), Some(0));
        assert_eq!(m.find(&30), Some(1));
        assert_eq!(m.find(&40), Some(2));
        assert_eq!(m.get(&40), Some(&80));
        m.erase(&999); // erasing end() is a no-op
        assert_eq!(m.size(), 3);
    }

    #[test]
    fn get_or_default_matches_cpp_index_operator() {
        let mut m: InsertionOrderedMap<i32, i32> = InsertionOrderedMap::new();
        *m.get_or_default(5) = 50;
        assert_eq!(m.get(&5), Some(&50));
        *m.get_or_default(5) += 1; // existing: no new entry
        assert_eq!(m.get(&5), Some(&51));
        assert_eq!(m.size(), 1);
    }
}
