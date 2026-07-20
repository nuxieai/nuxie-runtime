//! Inline port of `luau/tests/DenseHash.test.cpp` (`TEST_SUITE("DenseHashTests")`).
//! Behavioral tests for `DenseHashMap`: overwriting at the rehash threshold and
//! merging two maps. The upstream cases exercise C++ iterator-invalidation
//! during mutation (iterating a map while writing to it); Rust's borrow checker
//! forbids that pattern, so each snapshots the keys first and then mutates —
//! producing the identical final map state, which is what these assert on.

#![cfg(test)]

use crate::records::dense_hash_map::DenseHashMap;

#[test]
fn overwriting_an_existing_field_when_full_shouldnt_rehash() {
    let mut m: DenseHashMap<i32, i32> = DenseHashMap::new(-1);
    for i in 0..12 {
        *m.get_or_insert(i) = i;
    }
    assert_eq!(m.size(), 12);

    let keys: Vec<i32> = m.iter().map(|(k, _)| *k).collect();
    for k in keys {
        let a = *m.find(&k).expect("key present");
        *m.get_or_insert(k) = a + 1;
    }

    for i in 0..m.size() as i32 {
        let a = m.find(&i).expect("key present");
        assert_eq!(i + 1, *a);
    }
}

#[test]
fn merging_another_map_and_resolve_conflicts_that_also_just_so_happens_to_rehash_while_iterating() {
    let mut m1: DenseHashMap<i32, i32> = DenseHashMap::new(-1);
    for i in 0..12 {
        *m1.get_or_insert(i) = i;
    }

    let mut m2: DenseHashMap<i32, i32> = DenseHashMap::new(-1);
    for i in 8..24 {
        *m2.get_or_insert(i) = i;
    }

    assert_eq!(m1.size(), 12);
    assert_eq!(m2.size(), 16);

    // m1[k] += m2[k] for every shared key.
    let m1_keys: Vec<i32> = m1.iter().map(|(k, _)| *k).collect();
    for k in m1_keys {
        if let Some(b) = m2.find(&k).copied() {
            let a = *m1.find(&k).expect("key present");
            *m1.get_or_insert(k) = a + b;
        }
    }

    // Copy in every m2 key that m1 doesn't already have.
    let m2_keys: Vec<i32> = m2.iter().map(|(k, _)| *k).collect();
    for k in m2_keys {
        if m1.find(&k).is_none() {
            let a = *m2.find(&k).expect("key present");
            *m1.get_or_insert(k) = a;
        }
    }

    assert_eq!(m1.size(), 24);
    for i in 0..m1.size() as i32 {
        let a = *m1.find(&i).expect("key present");
        if i < 8 || i >= 12 {
            assert_eq!(i, a);
        } else {
            assert_eq!(i + i, a);
        }
    }
}
