//! Presence-bit dense hash table from `Luau/DenseHash2.h`.
//!
//! Unlike `DenseHashTable`, this version has no sentinel key and supports
//! erasure. It uses Fibonacci hashing, linear probing, and Algorithm R
//! backward-shift deletion.

use alloc::vec::Vec;
use core::marker::PhantomData;
use core::mem;
use core::slice;

use crate::functions::count_trailing_zeroes::count_trailing_zeroes;
use crate::records::bit_set::BitSet;
use crate::records::dense_hash_table::{DenseDefault, DenseEq, DenseHasher};

pub trait ItemInterface2<K, I> {
    fn get_key(item: &I) -> &K;
    fn set_key(item: &mut I, key: K);
    fn make_empty() -> I;
}

pub struct ItemInterfaceSet2<K>(PhantomData<K>);

impl<K: DenseDefault> ItemInterface2<K, K> for ItemInterfaceSet2<K> {
    fn get_key(item: &K) -> &K {
        item
    }

    fn set_key(item: &mut K, key: K) {
        *item = key;
    }

    fn make_empty() -> K {
        K::dense_default()
    }
}

pub struct ItemInterfaceMap2<K, V>(PhantomData<(K, V)>);

impl<K: DenseDefault, V: DenseDefault> ItemInterface2<K, (K, V)> for ItemInterfaceMap2<K, V> {
    fn get_key(item: &(K, V)) -> &K {
        &item.0
    }

    fn set_key(item: &mut (K, V), key: K) {
        item.0 = key;
    }

    fn make_empty() -> (K, V) {
        (K::dense_default(), V::dense_default())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BucketResult {
    pub(crate) bucket: usize,
    pub(crate) found: bool,
}

pub struct DenseHashTable2<K, I, Iface, H, E> {
    pub(crate) data: Vec<I>,
    used_table: BitSet,
    capacity: usize,
    count: usize,
    hash_shift: u8,
    hasher: H,
    eq: E,
    _key: PhantomData<K>,
    _iface: PhantomData<Iface>,
}

impl<K, I, Iface, H, E> Clone for DenseHashTable2<K, I, Iface, H, E>
where
    I: Clone,
    H: Clone,
    E: Clone,
{
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            used_table: self.used_table.clone(),
            capacity: self.capacity,
            count: self.count,
            hash_shift: self.hash_shift,
            hasher: self.hasher.clone(),
            eq: self.eq.clone(),
            _key: PhantomData,
            _iface: PhantomData,
        }
    }
}

impl<K, I, Iface, H, E> DenseHashTable2<K, I, Iface, H, E>
where
    K: Clone,
    Iface: ItemInterface2<K, I>,
    H: DenseHasher<K> + Default,
    E: DenseEq<K> + Default,
{
    pub fn new(buckets: usize) -> Self {
        debug_assert_eq!(buckets & buckets.wrapping_sub(1), 0);

        let data = (0..buckets).map(|_| Iface::make_empty()).collect();
        let hash_shift = if buckets == 0 {
            64
        } else {
            (64 - count_trailing_zeroes(buckets as u64)) as u8
        };

        Self {
            data,
            used_table: BitSet::new(buckets),
            capacity: buckets,
            count: 0,
            hash_shift,
            hasher: H::default(),
            eq: E::default(),
            _key: PhantomData,
            _iface: PhantomData,
        }
    }

    pub fn clear(&mut self, threshold_to_destroy: usize) {
        if self.count == 0 {
            return;
        }

        if self.capacity > threshold_to_destroy {
            self.destroy();
        } else {
            for item in &mut self.data {
                *item = Iface::make_empty();
            }
            self.used_table.clear();
        }

        self.count = 0;
    }

    pub(crate) fn destroy(&mut self) {
        self.data.clear();
        self.used_table = BitSet::default();
        self.capacity = 0;
        self.hash_shift = 64;
    }

    fn do_hash(&self, key: &K) -> usize {
        ((self.hasher.hash(key) as u64).wrapping_mul(11_400_714_819_323_198_485) >> self.hash_shift)
            as usize
    }

    pub(crate) fn erase(&mut self, key: &K) {
        if self.count == 0 {
            return;
        }

        let result = self.get_bucket(key);
        if result.found {
            self.do_erase(result.bucket);
        }
    }

    pub(crate) fn insert_unsafe(&mut self, key: K) -> usize {
        let result = self.get_bucket(&key);

        if !result.found {
            self.used_table.set(result.bucket, true);
            Iface::set_key(&mut self.data[result.bucket], key);
            self.count += 1;
        }

        result.bucket
    }

    pub(crate) fn find(&self, key: &K) -> Option<usize> {
        if self.count == 0 {
            return None;
        }

        let result = self.get_bucket(key);
        result.found.then_some(result.bucket)
    }

    pub(crate) fn grow(&mut self) {
        let new_size = if self.capacity == 0 {
            16
        } else {
            self.capacity * 2
        };
        let mut new_table = Self::new(new_size);

        for word_index in 0..self.used_table.num_words() {
            let mut word = self.used_table.word_at(word_index);
            while word != 0 {
                let bit = count_trailing_zeroes(word);
                let bucket = word_index * BitSet::NUM_ELEMENTS + bit;
                let key = Iface::get_key(&self.data[bucket]).clone();
                let target = new_table.insert_unsafe(key);
                new_table.data[target] = mem::replace(&mut self.data[bucket], Iface::make_empty());
                word &= word - 1;
            }
        }

        debug_assert_eq!(self.count, new_table.count);
        mem::swap(&mut self.data, &mut new_table.data);
        mem::swap(&mut self.used_table, &mut new_table.used_table);
        mem::swap(&mut self.capacity, &mut new_table.capacity);
        mem::swap(&mut self.hash_shift, &mut new_table.hash_shift);
    }

    pub(crate) fn rehash_if_full(&mut self, key: &K) {
        if self.count >= self.capacity * 3 / 4 && self.find(key).is_none() {
            self.grow();
        }
    }

    pub fn size(&self) -> usize {
        self.count
    }

    pub fn iter(&self) -> DenseHashTable2Iter<'_, I> {
        DenseHashTable2Iter {
            used_table: &self.used_table,
            inner: self.data.iter().enumerate(),
        }
    }

    pub fn iter_mut(&mut self) -> DenseHashTable2IterMut<'_, I> {
        DenseHashTable2IterMut {
            used_table: &self.used_table,
            inner: self.data.iter_mut().enumerate(),
        }
    }

    fn get_bucket(&self, key: &K) -> BucketResult {
        debug_assert!(self.count < self.capacity);
        let hash_mod = self.capacity - 1;
        let mut bucket = self.do_hash(key);

        loop {
            if !self.used_table.contains(bucket) {
                return BucketResult {
                    bucket,
                    found: false,
                };
            }
            if self.eq.eq(Iface::get_key(&self.data[bucket]), key) {
                return BucketResult {
                    bucket,
                    found: true,
                };
            }
            bucket = (bucket + 1) & hash_mod;
        }
    }

    fn do_erase(&mut self, bucket: usize) {
        let mut i = bucket;
        let mut j = bucket;
        let hash_mod = self.capacity - 1;

        loop {
            j = (j + 1) & hash_mod;
            if !self.used_table.contains(j) {
                break;
            }

            let r = self.do_hash(Iface::get_key(&self.data[j]));
            let left = i.wrapping_sub(r) & hash_mod;
            let right = j.wrapping_sub(r) & hash_mod;

            if left < right {
                self.data[i] = mem::replace(&mut self.data[j], Iface::make_empty());
                self.used_table.set(i, true);
                self.used_table.set(j, false);
                i = j;
            }
        }

        self.used_table.set(i, false);
        self.count -= 1;
    }
}

pub struct DenseHashTable2Iter<'a, I> {
    used_table: &'a BitSet,
    inner: core::iter::Enumerate<slice::Iter<'a, I>>,
}

impl<'a, I> Iterator for DenseHashTable2Iter<'a, I> {
    type Item = &'a I;

    fn next(&mut self) -> Option<Self::Item> {
        for (index, item) in self.inner.by_ref() {
            if self.used_table.contains(index) {
                return Some(item);
            }
        }
        None
    }
}

pub struct DenseHashTable2IterMut<'a, I> {
    used_table: &'a BitSet,
    inner: core::iter::Enumerate<slice::IterMut<'a, I>>,
}

impl<'a, I> Iterator for DenseHashTable2IterMut<'a, I> {
    type Item = &'a mut I;

    fn next(&mut self) -> Option<Self::Item> {
        for (index, item) in self.inner.by_ref() {
            if self.used_table.contains(index) {
                return Some(item);
            }
        }
        None
    }
}
