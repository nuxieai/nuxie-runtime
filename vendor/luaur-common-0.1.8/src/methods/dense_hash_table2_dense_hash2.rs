use core::marker::PhantomData;
use core::mem;

use crate::functions::count_trailing_zeroes::count_trailing_zeroes;
use crate::records::bit_set::BitSet;
use crate::records::bucket_result::BucketResult;
use crate::records::dense_hash_table::{DenseEq, DenseHasher};
use crate::records::dense_hash_table2::DenseHashTable2;
use crate::records::dense_hash_table2_const_iterator::DenseHashTable2ConstIterator;
use crate::records::dense_hash_table2_iterator::DenseHashTable2Iterator;
use crate::records::item_interface2::ItemInterface2;

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

        let data = (0..buckets).map(|_| None).collect();
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
            for bucket in self.used_table.iter() {
                self.data[bucket] = None;
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
            self.data[result.bucket] = Some(Iface::make(key));
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

        for bucket in self.used_table.iter() {
            let item = self.data[bucket].take().expect("occupied bucket");
            let result = new_table.get_bucket(Iface::get_key(&item));
            debug_assert!(!result.found);
            new_table.used_table.set(result.bucket, true);
            new_table.count += 1;
            new_table.data[result.bucket] = Some(item);
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

    pub fn iter(&self) -> DenseHashTable2ConstIterator<'_, I> {
        DenseHashTable2ConstIterator {
            used_table: &self.used_table,
            inner: self.data.iter().enumerate(),
        }
    }

    pub fn iter_mut(&mut self) -> DenseHashTable2Iterator<'_, I> {
        DenseHashTable2Iterator {
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
            if self.eq.eq(
                Iface::get_key(self.data[bucket].as_ref().expect("occupied bucket")),
                key,
            ) {
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

            let r = self.do_hash(Iface::get_key(
                self.data[j].as_ref().expect("occupied bucket"),
            ));
            let left = i.wrapping_sub(r) & hash_mod;
            let right = j.wrapping_sub(r) & hash_mod;

            if left < right {
                self.used_table.set(i, true);
                self.used_table.set(j, false);
                self.data[i] = self.data[j].take();
                i = j;
            }
        }

        self.used_table.set(i, false);
        self.data[i] = None;
        self.count -= 1;
    }
}

impl<'a, I> Iterator for DenseHashTable2ConstIterator<'a, I> {
    type Item = &'a I;

    fn next(&mut self) -> Option<Self::Item> {
        for (index, item) in self.inner.by_ref() {
            if self.used_table.contains(index) {
                return Some(item.as_ref().expect("occupied bucket"));
            }
        }
        None
    }
}

impl<'a, I> Iterator for DenseHashTable2Iterator<'a, I> {
    type Item = &'a mut I;

    fn next(&mut self) -> Option<Self::Item> {
        for (index, item) in self.inner.by_ref() {
            if self.used_table.contains(index) {
                return Some(item.as_mut().expect("occupied bucket"));
            }
        }
        None
    }
}
