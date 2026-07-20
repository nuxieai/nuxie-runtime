//! Inline port of `luau/tests/InsertionOrderedMap.test.cpp`
//! (`TEST_SUITE("InsertionOrderedMap")`). The upstream `MapFixture` hands out
//! stable `int*` keys; here a `Vec<Box<i32>>` plays the same role — each box's
//! heap address is a stable `*const i32` even as the vec reallocates.

#![cfg(test)]

use crate::records::insertion_ordered_map::InsertionOrderedMap;

#[derive(Default)]
struct MapFixture {
    ptrs: Vec<Box<i32>>,
}

impl MapFixture {
    fn make_ptr(&mut self) -> *const i32 {
        self.ptrs.push(Box::new(0));
        &**self.ptrs.last().unwrap() as *const i32
    }
}

#[test]
fn map_insertion() {
    let mut fx = MapFixture::default();
    let mut map: InsertionOrderedMap<*const i32, i32> = InsertionOrderedMap::new();

    let a = fx.make_ptr();
    let b = fx.make_ptr();

    map.insert(a, 1);
    map.insert(b, 2);
}

#[test]
fn map_lookup() {
    let mut fx = MapFixture::default();
    let mut map: InsertionOrderedMap<*const i32, i32> = InsertionOrderedMap::new();

    let a = fx.make_ptr();
    map.insert(a, 1);

    let r = map.get(&a);
    assert!(r.is_some());
    assert_eq!(*r.unwrap(), 1);

    let missing = fx.make_ptr();
    assert!(map.get(&missing).is_none());
}

#[test]
fn insert_does_not_update() {
    let mut fx = MapFixture::default();
    let mut map: InsertionOrderedMap<*const i32, i32> = InsertionOrderedMap::new();

    let k = fx.make_ptr();
    map.insert(k, 1);
    map.insert(k, 2);

    let v = map.get(&k);
    assert!(v.is_some());
    assert_eq!(*v.unwrap(), 1);
}

#[test]
fn insertion_order_is_iteration_order() {
    let mut fx = MapFixture::default();
    let mut map: InsertionOrderedMap<*const i32, i32> = InsertionOrderedMap::new();

    let a = fx.make_ptr();
    let b = fx.make_ptr();
    let c = fx.make_ptr();
    map.insert(a, 1);
    map.insert(b, 1);
    map.insert(c, 1);

    let items: Vec<(*const i32, i32)> = map.iter().copied().collect();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0], (a, 1));
    assert_eq!(items[1], (b, 1));
    assert_eq!(items[2], (c, 1));
}

#[test]
fn destructuring_iterator_compiles() {
    // Upstream: this test's only purpose is to compile (an empty map's iterator).
    let map: InsertionOrderedMap<*const i32, i32> = InsertionOrderedMap::new();
    for &(k, v) in map.iter() {
        assert!(!k.is_null());
        assert!(v > 0);
    }
}

#[test]
fn map_erasure() {
    let mut fx = MapFixture::default();
    let mut map: InsertionOrderedMap<*const i32, i32> = InsertionOrderedMap::new();

    let a = fx.make_ptr();
    let b = fx.make_ptr();

    map.insert(a, 1);
    map.insert(b, 2);

    map.erase(&a);
    assert_eq!(map.size(), 1);
    assert!(!map.contains(&a));
    assert!(map.get(&a).is_none());

    assert!(map.get(&b).is_some());
}

#[test]
fn map_clear() {
    let mut fx = MapFixture::default();
    let mut map: InsertionOrderedMap<*const i32, i32> = InsertionOrderedMap::new();

    let a = fx.make_ptr();
    map.insert(a, 1);

    map.clear();
    assert_eq!(map.size(), 0);
    assert!(!map.contains(&a));
    assert!(map.get(&a).is_none());
}
