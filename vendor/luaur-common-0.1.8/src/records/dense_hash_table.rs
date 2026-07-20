//! Faithful port of `Luau::detail::DenseHashTable` — the open-addressing core
//! shared by `DenseHashMap`/`DenseHashSet`. Reference:
//! `luau/Common/include/Luau/DenseHash.h` (open addressing, quadratic probing,
//! empty-key sentinel). Oracle: the two upstream cases in
//! `luau/tests/DenseHash.test.cpp` + a `std::collections::HashMap` differential
//! fuzz (2000 trials × 200 ops). The validated standalone prototype this file
//! transcribes lives at `/tmp/densehash_proto.rs`.
//!
//! Design (option A — generic functor params): the C++ `Hash`/`Eq` template
//! parameters become the `DenseHasher`/`DenseEq` traits, and the `Set`/`Map`
//! item layout becomes the `ItemInterface` trait. The table stores items in a
//! `Vec<I>` and returns slot indices (`usize`) rather than raw pointers, which
//! keeps the port sound without `unsafe`. `core`/`alloc` only, so the crate
//! stays `wasm32-unknown-unknown` compatible.

use alloc::vec::Vec;
use core::marker::PhantomData;
use core::mem;

// ---- functor traits (C++ `Hash` / `Eq` template params) ----

/// Hash functor, mirroring the C++ `Hash` template parameter.
pub trait DenseHasher<K> {
    fn hash(&self, key: &K) -> usize;
}

/// Equality functor, mirroring the C++ `Eq` template parameter.
pub trait DenseEq<K> {
    fn eq(&self, a: &K, b: &K) -> bool;
}

/// Default equality functor used when none is supplied (`std::equal_to<T>`).
#[derive(Clone, Copy)]
pub struct DenseEqDefault<K>(PhantomData<K>);

// Manual impl: the derive would demand `K: Default` (PhantomData needs nothing).
impl<K> Default for DenseEqDefault<K> {
    fn default() -> Self {
        DenseEqDefault(PhantomData)
    }
}

impl<K: PartialEq> DenseEq<K> for DenseEqDefault<K> {
    fn eq(&self, a: &K, b: &K) -> bool {
        a == b
    }
}

// ---- empty-slot value initialization ----

/// Empty-slot value for a `DenseHashMap`, mirroring C++ value-initialization: a
/// freshly grown bucket holds a value-initialized `Value()` — null for pointers,
/// zero for scalars, `Default::default()` for ordinary types. Rust's `Default`
/// is *not* implemented for raw pointers, and the orphan rule forbids adding it,
/// so a blanket `impl<T: Default>` cannot coexist with the pointer impls below
/// (coherence). Hence an explicit trait: pointer and scalar value types are
/// covered here, and any struct value type a map stores supplies its own impl
/// (e.g. `Location` in `luau-ast`).
pub trait DenseDefault {
    fn dense_default() -> Self;
}

impl<T> DenseDefault for *mut T {
    fn dense_default() -> Self {
        core::ptr::null_mut()
    }
}

impl<T> DenseDefault for *const T {
    fn dense_default() -> Self {
        core::ptr::null()
    }
}

macro_rules! dense_default_via_default {
    ($($t:ty),* $(,)?) => {
        $(
            impl DenseDefault for $t {
                fn dense_default() -> Self {
                    <$t as Default>::default()
                }
            }
        )*
    };
}

dense_default_via_default!(
    bool, i8, i16, i32, i64, isize, u8, u16, u32, u64, usize, f32, f64, char,
);

impl DenseDefault for alloc::string::String {
    fn dense_default() -> Self {
        alloc::string::String::new()
    }
}

impl<T> DenseDefault for alloc::vec::Vec<T> {
    fn dense_default() -> Self {
        alloc::vec::Vec::new()
    }
}

// A DenseHashMap value type can itself be a map/set (C++ stores
// `std::unordered_map`/`std::map`/`std::set` values, e.g.
// `DfgScope::Props = DenseHashMap<const Def*, std::unordered_map<...>>`), which
// default-construct empty.
impl<K: Ord, V> DenseDefault for alloc::collections::BTreeMap<K, V> {
    fn dense_default() -> Self {
        alloc::collections::BTreeMap::new()
    }
}

impl<T: Ord> DenseDefault for alloc::collections::BTreeSet<T> {
    fn dense_default() -> Self {
        alloc::collections::BTreeSet::new()
    }
}

// A nullable shared pointer (C++ `std::shared_ptr<T>`) defaults to null; the
// Rust mirror `Option<Arc<T>>` defaults to `None`. Used for DenseHashMap values
// whose C++ type is a smart pointer (e.g. `DenseHashMap<K, ScopePtr>`).
impl<T> DenseDefault for Option<T> {
    fn dense_default() -> Self {
        None
    }
}

impl<A: DenseDefault, B: DenseDefault> DenseDefault for (A, B) {
    fn dense_default() -> Self {
        (A::dense_default(), B::dense_default())
    }
}

// ---- item interface (Set: `I = K` ; Map: `I = (K, V)`) ----

/// Bridges the `Set`/`Map` item layout so the table is agnostic to whether a
/// slot stores a bare key or a `(key, value)` pair.
pub trait ItemInterface<K, I> {
    fn get_key(item: &I) -> &K;
    fn set_key(item: &mut I, key: K);
    fn make_empty(empty_key: &K) -> I;
}

/// Set layout: the item *is* the key.
pub struct ItemInterfaceSet<K>(PhantomData<K>);

impl<K: Clone> ItemInterface<K, K> for ItemInterfaceSet<K> {
    fn get_key(item: &K) -> &K {
        item
    }
    fn set_key(item: &mut K, key: K) {
        *item = key;
    }
    fn make_empty(empty_key: &K) -> K {
        empty_key.clone()
    }
}

/// Map layout: the item is a `(key, value)` pair; the value defaults on insert.
pub struct ItemInterfaceMap<K, V>(PhantomData<(K, V)>);

impl<K: Clone, V: DenseDefault> ItemInterface<K, (K, V)> for ItemInterfaceMap<K, V> {
    fn get_key(item: &(K, V)) -> &K {
        &item.0
    }
    fn set_key(item: &mut (K, V), key: K) {
        item.0 = key;
    }
    fn make_empty(empty_key: &K) -> (K, V) {
        (empty_key.clone(), V::dense_default())
    }
}

// ---- the table ----

/// `capacity == data.len()`, always a power of two or 0. A slot is empty iff its
/// key compares equal to `empty_key`.
pub struct DenseHashTable<K, I, Iface, H, E> {
    pub(crate) data: Vec<I>,
    pub(crate) capacity: usize,
    pub(crate) count: usize,
    pub(crate) empty_key: K,
    pub(crate) hasher: H,
    pub(crate) eq: E,
    pub(crate) _iface: PhantomData<Iface>,
}

// The C++ container is copyable; we provide `Clone`/`Debug` by hand (rather than
// `derive`) so the bounds stay precise — `derive` would spuriously require
// `Iface: Clone`/`Debug` on the zero-sized `PhantomData<Iface>` marker, and
// `Debug` would force `H`/`E` (the hash/eq functors) to be `Debug`.
impl<K: Clone, I: Clone, Iface, H: Clone, E: Clone> Clone for DenseHashTable<K, I, Iface, H, E> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            capacity: self.capacity,
            count: self.count,
            empty_key: self.empty_key.clone(),
            hasher: self.hasher.clone(),
            eq: self.eq.clone(),
            _iface: PhantomData,
        }
    }
}

impl<K: core::fmt::Debug, I: core::fmt::Debug, Iface, H, E> core::fmt::Debug
    for DenseHashTable<K, I, Iface, H, E>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DenseHashTable")
            .field("data", &self.data)
            .field("capacity", &self.capacity)
            .field("count", &self.count)
            .field("empty_key", &self.empty_key)
            .finish_non_exhaustive()
    }
}

impl<K, I, Iface, H, E> DenseHashTable<K, I, Iface, H, E>
where
    K: Clone,
    Iface: ItemInterface<K, I>,
    H: DenseHasher<K> + Default,
    E: DenseEq<K> + Default,
{
    /// `DenseHashTable(const Key& empty_key, size_t buckets)`. `buckets` must be
    /// a power of two or 0. Reference: `DenseHash.h` ctor.
    pub fn new(empty_key: K, buckets: usize) -> Self {
        let eq = E::default();
        // equality must at least recognise the sentinel as equal to itself
        debug_assert!(eq.eq(&empty_key, &empty_key));
        debug_assert!(buckets & buckets.wrapping_sub(1) == 0);

        let data = if buckets > 0 {
            (0..buckets)
                .map(|_| Iface::make_empty(&empty_key))
                .collect()
        } else {
            Vec::new()
        };

        DenseHashTable {
            data,
            capacity: buckets,
            count: 0,
            empty_key,
            hasher: H::default(),
            eq,
            _iface: PhantomData,
        }
    }

    /// Inserts `key` (or finds it if present) without checking load factor, and
    /// returns the slot index. The caller must `rehash_if_full` first.
    pub(crate) fn insert_unsafe(&mut self, key: K) -> usize {
        debug_assert!(!self.eq.eq(&key, &self.empty_key));
        let hashmod = self.capacity - 1;
        let mut bucket = self.hasher.hash(&key) & hashmod;
        for probe in 0..=hashmod {
            if self
                .eq
                .eq(Iface::get_key(&self.data[bucket]), &self.empty_key)
            {
                Iface::set_key(&mut self.data[bucket], key);
                self.count += 1;
                return bucket;
            }
            if self.eq.eq(Iface::get_key(&self.data[bucket]), &key) {
                return bucket;
            }
            bucket = (bucket + probe + 1) & hashmod;
        }
        let occupied = self
            .data
            .iter()
            .filter(|item| !self.eq.eq(Iface::get_key(item), &self.empty_key))
            .count();
        unreachable!(
            "dense hash table is full: capacity={}, count={}, occupied={}",
            self.capacity, self.count, occupied
        );
    }

    /// Returns the slot index of `key` if present.
    pub(crate) fn find(&self, key: &K) -> Option<usize> {
        if self.count == 0 {
            return None;
        }
        if self.eq.eq(key, &self.empty_key) {
            return None;
        }
        let hashmod = self.capacity - 1;
        let mut bucket = self.hasher.hash(key) & hashmod;
        for probe in 0..=hashmod {
            let k = Iface::get_key(&self.data[bucket]);
            if self.eq.eq(k, key) {
                return Some(bucket);
            }
            if self.eq.eq(k, &self.empty_key) {
                return None;
            }
            bucket = (bucket + probe + 1) & hashmod;
        }
        let occupied = self
            .data
            .iter()
            .filter(|item| !self.eq.eq(Iface::get_key(item), &self.empty_key))
            .count();
        unreachable!(
            "dense hash table is full: capacity={}, count={}, occupied={}",
            self.capacity, self.count, occupied
        );
    }

    /// Grows to `16` (from empty) or `2×` capacity, re-inserting live items.
    pub(crate) fn rehash(&mut self) {
        let newsize = if self.capacity == 0 {
            16
        } else {
            self.capacity * 2
        };
        let mut newtable = Self::new(self.empty_key.clone(), newsize);
        for i in 0..self.capacity {
            if !self.eq.eq(Iface::get_key(&self.data[i]), &self.empty_key) {
                let key = Iface::get_key(&self.data[i]).clone();
                let idx = newtable.insert_unsafe(key);
                newtable.data[idx] =
                    mem::replace(&mut self.data[i], Iface::make_empty(&self.empty_key));
            }
        }
        debug_assert_eq!(self.count, newtable.count);
        mem::swap(&mut self.data, &mut newtable.data);
        mem::swap(&mut self.capacity, &mut newtable.capacity);
    }

    /// Rehashes before an insert that would push past the 3/4 load factor — but
    /// only when `key` is genuinely new. The `find` guard is load-bearing:
    /// overwriting an existing key when full must NOT rehash (upstream relies on
    /// iterators surviving an overwrite-merge).
    pub(crate) fn rehash_if_full(&mut self, key: &K) {
        if self.count >= self.capacity * 3 / 4 && self.find(key).is_none() {
            self.rehash();
        }
    }

    /// Clears all slots back to the empty sentinel without freeing capacity.
    pub(crate) fn clear(&mut self) {
        if self.count == 0 {
            return;
        }
        for slot in self.data.iter_mut() {
            *slot = Iface::make_empty(&self.empty_key);
        }
        self.count = 0;
    }

    pub(crate) fn size(&self) -> usize {
        self.count
    }

    /// Index of the first occupied slot (== `capacity` when empty). Iterator seed.
    pub(crate) fn first_occupied(&self) -> usize {
        let mut start = 0;
        while start < self.capacity
            && self
                .eq
                .eq(Iface::get_key(&self.data[start]), &self.empty_key)
        {
            start += 1;
        }
        start
    }

    /// Index of the next occupied slot strictly after `index`.
    pub(crate) fn next_occupied(&self, mut index: usize) -> usize {
        loop {
            index += 1;
            if index >= self.capacity
                || !self
                    .eq
                    .eq(Iface::get_key(&self.data[index]), &self.empty_key)
            {
                break;
            }
        }
        index
    }
}
