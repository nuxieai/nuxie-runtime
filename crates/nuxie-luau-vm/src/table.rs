//! Tables (plan decision D4).
//!
//! The array part is ported from C++ Luau exactly, because eleven
//! conformance sites pin it: `computesizes`, `adjustasize` with `aextra`,
//! the `sizearray+1` rehash trigger, `resizearray` for `SETLIST`,
//! `luaH_getn`'s last-slot fast path and binary search. The `aboundary`
//! cache is left out (D4 showed it unobservable).
//!
//! The hash part is C++'s chained scatter table with Brent's variation
//! (`luaH_newkey`, `getfreepos`) on indices: `next` is a signed offset to
//! the next node of the chain. Its hash function is this crate's own, so
//! `pairs` order differs from C++ (three conformance sites, filed as
//! expected-fail by D4). A deleted key keeps its node with a nil value, so
//! `next` accepts it as a cursor, and deletion never moves an entry.
//!
//! Every access is panic-free: positions come from masking or checked
//! `get`, and the unreachable branches return the absent answer.
//!
//! The table library (`libs/table.rs`) builds on `new`, `resize_array`,
//! `len`, `next_from`, `clear` and `max_number_key` here.

use crate::budget::Budget;
use crate::value::{StrId, TableId, Value};

const MAXBITS: usize = 26;
/// Largest array or hash part (C++ `MAXSIZE`).
pub(crate) const MAXSIZE: usize = 1 << MAXBITS;

#[derive(Clone, Copy, Default)]
pub(crate) struct Node {
    pub(crate) key: Value,
    pub(crate) val: Value,
    /// Offset to the next node of this chain, 0 at the end.
    next: i32,
}

#[derive(Clone, Default)]
pub(crate) struct Table {
    /// Private, with `nodes`: the parts grow only through the methods that
    /// take a `Budget` (budget.rs, rule 2). `array_mut` lends a slice,
    /// which cannot grow.
    array: Vec<Value>,
    nodes: Vec<Node>,
    /// C++ `lastfree`: every node at or above it has been handed out.
    lastfree: usize,
    pub(crate) meta: Option<TableId>,
    pub(crate) readonly: bool,
    /// C++ `safeenv`: imports and builtin fast paths may assume the
    /// environment's library functions are the original ones.
    pub(crate) safeenv: bool,
}

/// Why a raw store failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TableError {
    NilIndex,
    NanIndex,
    /// A vector key with a NaN component (`luaH_newkey`).
    VectorNanIndex,
    /// The table would exceed `MAXSIZE`.
    Overflow,
    /// The growth would pass the memory budget.
    Memory,
}

#[inline]
fn mix(x: u64) -> u64 {
    // splitmix64 finalizer
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[inline]
fn hash_str(id: StrId) -> usize {
    // Fibonacci hashing of the interned id: one multiply, no memory access.
    (u64::from(id.0).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) as usize
}

#[inline]
fn hash_value(key: Value) -> usize {
    match key {
        Value::String(id) => hash_str(id),
        Value::Number(n) => {
            // -0 and 0 are the same key.
            let n = if n == 0.0 { 0.0 } else { n };
            mix(n.to_bits()) as usize
        }
        // C++ `hashint` exactly: lookups compare integer keys as doubles
        // (`raw_eq`), so whether two integers equal as doubles find each
        // other depends on this hash, as it does upstream.
        Value::Integer(i) => hash_int(i),
        Value::Boolean(b) => usize::from(b).wrapping_add(1),
        Value::Table(id) => mix(u64::from(id.0) | (1 << 40)) as usize,
        Value::Function(id) => mix(u64::from(id.0) | (2 << 40)) as usize,
        Value::UpRef(id) => mix(u64::from(id.0) | (5 << 40)) as usize,
        Value::Thread(id) => mix(u64::from(id.0) | (6 << 40)) as usize,
        Value::Buffer(id) => mix(u64::from(id.0) | (7 << 40)) as usize,
        Value::LightUserdata(p, tag) => mix(p ^ (u64::from(tag) << 56)) as usize,
        Value::Vector(v) => {
            let mut h = 0u64;
            for c in v {
                let c = if c == 0.0 { 0.0 } else { c };
                h = mix(h ^ u64::from(c.to_bits()));
            }
            h as usize
        }
        Value::Nil => 0,
    }
}

/// C++ `hashint`: the MurmurHash64B finalizer's low half.
#[inline]
fn hash_int(n: i64) -> usize {
    const M: u32 = 0x5bd1_e995;
    let bits = n as u64;
    let mut h1 = bits as u32;
    let mut h2 = (bits >> 32) as u32;
    h1 ^= h2 >> 18;
    h1 = h1.wrapping_mul(M);
    h2 ^= h1 >> 22;
    h2 = h2.wrapping_mul(M);
    h1 ^= h2 >> 17;
    h1 = h1.wrapping_mul(M);
    h2 ^= h1 >> 19;
    h2 = h2.wrapping_mul(M);
    h2 as usize
}

/// C++ `arrayindex`: the 1-based array index a number key would have, if it
/// is an integer that fits in an `int`.
#[inline]
pub(crate) fn array_index(key: Value) -> Option<i64> {
    match key {
        Value::Number(n) => {
            let i = n as i32;
            if f64::from(i) == n {
                Some(i64::from(i))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// The 0-based array slot for a 1-based index, if it is inside `len`.
#[inline]
fn array_slot(index: i64, len: usize) -> Option<usize> {
    let slot = usize::try_from(index.checked_sub(1)?).ok()?;
    if slot < len { Some(slot) } else { None }
}

fn ceillog2(k: usize) -> usize {
    let mut log = 0usize;
    while log < usize::BITS as usize && 1usize.checked_shl(log as u32).is_some_and(|p| p < k) {
        log = log.saturating_add(1);
    }
    log
}

fn next_pow2(n: usize) -> usize {
    if n == 0 {
        0
    } else {
        n.checked_next_power_of_two().unwrap_or(MAXSIZE)
    }
}

/// The signed node offset from `from` to `to` (both below `MAXSIZE`).
#[inline]
fn offset(to: usize, from: usize) -> i32 {
    (to as i64).wrapping_sub(from as i64) as i32
}

#[inline]
fn step(index: usize, delta: i32) -> usize {
    index.wrapping_add_signed(delta as isize)
}

/// Approximate bytes of one hash node.
const NODE_BYTES: usize = 40;

#[inline]
fn normalize_key(key: Value) -> Value {
    match key {
        // `-0.0 + 0.0` is `+0.0` and every other number is unchanged, so both
        // zeros hash to one node (C++ `luai_hashnum` folds them the same way).
        Value::Number(n) => Value::Number(n + 0.0),
        other => other,
    }
}

impl Table {
    /// C++ `luaH_new(narray, nhash)`. Only `Heap::new_table` calls this,
    /// after charging `capacity_for(narray, nhash)`.
    pub(crate) fn new(narray: usize, nhash: usize) -> Table {
        let mut table = Table {
            array: vec![Value::Nil; narray.min(MAXSIZE)],
            ..Table::default()
        };
        table.set_sizenode(next_pow2(nhash.min(MAXSIZE)));
        table
    }

    /// Bytes of storage, for the heap accounting.
    /// The bytes a hash part sized for `n` keys takes.
    pub(crate) fn node_bytes(n: usize) -> usize {
        n.checked_next_power_of_two()
            .unwrap_or(usize::MAX)
            .saturating_mul(NODE_BYTES)
    }

    /// The capacity `new(narray, nhash)` allocates.
    pub(crate) fn capacity_for(narray: usize, nhash: usize) -> usize {
        narray
            .min(MAXSIZE)
            .saturating_mul(16)
            .saturating_add(next_pow2(nhash.min(MAXSIZE)).saturating_mul(NODE_BYTES))
    }

    /// The array part, read-only.
    #[inline]
    pub(crate) fn array(&self) -> &[Value] {
        &self.array
    }

    /// The array part as a slice: its values may change, its size cannot.
    #[inline]
    pub(crate) fn array_mut(&mut self) -> &mut [Value] {
        &mut self.array
    }

    pub(crate) fn capacity_bytes(&self) -> usize {
        self.array
            .len()
            .saturating_mul(16)
            .saturating_add(self.nodes.len().saturating_mul(NODE_BYTES))
    }

    fn set_sizenode(&mut self, n: usize) {
        self.nodes = vec![Node::default(); n];
        self.lastfree = n;
    }

    #[inline]
    fn main_position(&self, key: Value) -> usize {
        hash_value(key) & self.nodes.len().wrapping_sub(1)
    }

    #[inline]
    fn find_node(&self, key: Value) -> Option<usize> {
        if self.nodes.is_empty() {
            return None;
        }
        let mut i = self.main_position(key);
        loop {
            let node = self.nodes.get(i)?;
            if node.key.raw_eq(key) {
                return Some(i);
            }
            if node.next == 0 {
                return None;
            }
            i = step(i, node.next);
        }
    }

    #[inline]
    fn find_str(&self, key: StrId) -> Option<usize> {
        if self.nodes.is_empty() {
            return None;
        }
        let mut i = hash_str(key) & self.nodes.len().wrapping_sub(1);
        loop {
            let node = self.nodes.get(i)?;
            if let Value::String(s) = node.key
                && s == key
            {
                return Some(i);
            }
            if node.next == 0 {
                return None;
            }
            i = step(i, node.next);
        }
    }

    #[inline]
    pub(crate) fn get_str(&self, key: StrId) -> Value {
        self.find_str(key)
            .and_then(|i| self.nodes.get(i))
            .map_or(Value::Nil, |n| n.val)
    }

    #[inline]
    pub(crate) fn get_int(&self, index: i64) -> Value {
        if let Some(slot) = array_slot(index, self.array.len()) {
            return self.array.get(slot).copied().unwrap_or_default();
        }
        self.get_hash(Value::Number(index as f64))
    }

    #[inline]
    fn get_hash(&self, key: Value) -> Value {
        self.find_node(key)
            .and_then(|i| self.nodes.get(i))
            .map_or(Value::Nil, |n| n.val)
    }

    /// Raw get (C++ `luaH_get`).
    #[inline]
    pub(crate) fn get(&self, key: Value) -> Value {
        match key {
            Value::String(s) => self.get_str(s),
            Value::Number(_) => match array_index(key) {
                Some(i) => self.get_int(i),
                None => self.get_hash(key),
            },
            Value::Nil => Value::Nil,
            _ => self.get_hash(key),
        }
    }

    /// Raw set (C++ `luaH_set` followed by the store). As in C++, a missing
    /// key is created even when the value is nil; that is what drives the
    /// `sizearray+1` rehash trigger D4 pins.
    ///
    /// Growth is checked against `budget` before the table changes (a
    /// refused store leaves the table as it was) and settled after.
    pub(crate) fn set(
        &mut self,
        key: Value,
        val: Value,
        budget: &mut Budget,
    ) -> Result<(), TableError> {
        let before = self.capacity_bytes();
        if let Some(grown) = self.capacity_after_new_key(key) {
            budget
                .check(grown.saturating_sub(before))
                .map_err(|_| TableError::Memory)?;
        }
        let result = self.set_unaccounted(key, val);
        budget.settle(before, self.capacity_bytes());
        result
    }

    fn set_unaccounted(&mut self, key: Value, val: Value) -> Result<(), TableError> {
        if let Some(i) = array_index(key)
            && let Some(slot) = array_slot(i, self.array.len()).and_then(|s| self.array.get_mut(s))
        {
            *slot = val;
            return Ok(());
        }
        match key {
            Value::Nil => return Err(TableError::NilIndex),
            Value::Number(n) if n.is_nan() => return Err(TableError::NanIndex),
            Value::Vector(v) if v.iter().any(|c| c.is_nan()) => {
                return Err(TableError::VectorNanIndex);
            }
            _ => {}
        }
        let key = normalize_key(key);
        if let Some(node) = self.find_node(key).and_then(|i| self.nodes.get_mut(i)) {
            node.val = val;
            return Ok(());
        }
        self.newkey(key, val)
    }

    /// C++ `luaH_newkey`: the boundary-invariant trigger, then a free node
    /// or a rehash.
    fn newkey(&mut self, key: Value, val: Value) -> Result<(), TableError> {
        let next_array = (self.array.len() as i64).saturating_add(1);
        if array_index(key) == Some(next_array) || !self.insert_node(key, val) {
            self.rehash(Some(key))?;
            return self.array_or_newkey(key, val);
        }
        Ok(())
    }

    /// The tail of `luaH_set` after a rehash: the key now goes to the array
    /// part or a node (a rehash always leaves room for it).
    fn array_or_newkey(&mut self, key: Value, val: Value) -> Result<(), TableError> {
        if let Some(i) = array_index(key)
            && let Some(slot) = array_slot(i, self.array.len()).and_then(|s| self.array.get_mut(s))
        {
            *slot = val;
            return Ok(());
        }
        if let Some(node) = self.find_node(key).and_then(|i| self.nodes.get_mut(i)) {
            node.val = val;
            return Ok(());
        }
        if !self.insert_node(key, val) {
            // Only reachable if the sizing undercounted; grow geometrically.
            let keep = std::mem::take(&mut self.nodes);
            let size = keep.len().saturating_mul(2).max(1);
            if size > MAXSIZE {
                return Err(TableError::Overflow);
            }
            self.set_sizenode(size);
            for node in keep.into_iter().rev() {
                if !node.key.is_nil() && !node.val.is_nil() {
                    self.insert_node(node.key, node.val);
                }
            }
            self.insert_node(key, val);
        }
        Ok(())
    }

    /// C++ `getfreepos`.
    fn free_pos(&mut self) -> Option<usize> {
        while self.lastfree > 0 {
            self.lastfree = self.lastfree.saturating_sub(1);
            if self
                .nodes
                .get(self.lastfree)
                .is_some_and(|n| n.key.is_nil())
            {
                return Some(self.lastfree);
            }
        }
        None
    }

    /// The placement half of `luaH_newkey` for a key known to be absent.
    /// Returns false when there is no free node (the caller rehashes).
    fn insert_node(&mut self, key: Value, val: Value) -> bool {
        if self.nodes.is_empty() {
            return false;
        }
        let mut mp = self.main_position(key);
        let taken = self.nodes.get(mp).is_some_and(|n| !n.val.is_nil());
        if taken {
            let Some(free) = self.free_pos() else {
                return false;
            };
            let collider_key = self.nodes.get(mp).map_or(Value::Nil, |n| n.key);
            let other = self.main_position(collider_key);
            if other != mp {
                // The colliding node is out of its main position: move it to
                // the free node and give `mp` to the new key.
                let mut prev = other;
                loop {
                    let Some(node) = self.nodes.get(prev) else {
                        return false;
                    };
                    let next = step(prev, node.next);
                    if next == mp || node.next == 0 {
                        break;
                    }
                    prev = next;
                }
                let moved = self.nodes.get(mp).copied().unwrap_or_default();
                if let Some(node) = self.nodes.get_mut(prev) {
                    node.next = offset(free, prev);
                }
                if let Some(node) = self.nodes.get_mut(free) {
                    *node = moved;
                    if moved.next != 0 {
                        node.next = moved.next.wrapping_add(offset(mp, free));
                    }
                }
                if let Some(node) = self.nodes.get_mut(mp) {
                    node.next = 0;
                    node.val = Value::Nil;
                }
            } else {
                // The colliding node is in its main position: chain the new
                // key in the free node after it.
                let mp_next = self.nodes.get(mp).map_or(0, |n| n.next);
                if let Some(node) = self.nodes.get_mut(free) {
                    node.next = if mp_next != 0 {
                        offset(step(mp, mp_next), free)
                    } else {
                        0
                    };
                }
                if let Some(node) = self.nodes.get_mut(mp) {
                    node.next = offset(free, mp);
                }
                mp = free;
            }
        }
        if let Some(node) = self.nodes.get_mut(mp) {
            node.key = key;
            node.val = val;
        }
        true
    }

    /// C++ `numusearray`.
    fn numusearray(&self, nums: &mut [usize; MAXBITS + 1]) -> usize {
        let mut ause = 0usize;
        let mut i = 1usize;
        let len = self.array.len();
        let mut ttlg = 1usize;
        for num in nums.iter_mut() {
            let mut lc = 0usize;
            let mut lim = ttlg;
            if lim > len {
                lim = len;
                if i > lim {
                    break;
                }
            }
            while i <= lim {
                if self
                    .array
                    .get(i.wrapping_sub(1))
                    .is_some_and(|v| !v.is_nil())
                {
                    lc = lc.saturating_add(1);
                }
                i = i.saturating_add(1);
            }
            *num = num.saturating_add(lc);
            ause = ause.saturating_add(lc);
            ttlg = ttlg.saturating_mul(2);
        }
        ause
    }

    /// C++ `countint`.
    fn countint(key: Value, nums: &mut [usize; MAXBITS + 1]) -> usize {
        match array_index(key) {
            Some(i) if i > 0 && (i as usize) <= MAXSIZE => {
                if let Some(n) = nums.get_mut(ceillog2(i as usize)) {
                    *n = n.saturating_add(1);
                }
                1
            }
            _ => 0,
        }
    }

    /// C++ `computesizes`.
    fn computesizes(nums: &[usize; MAXBITS + 1], narray: &mut usize) -> usize {
        let (mut a, mut na, mut n) = (0usize, 0usize, 0usize);
        let mut twotoi = 1usize;
        let mut i = 0usize;
        while twotoi / 2 < *narray {
            let ni = nums.get(i).copied().unwrap_or(0);
            if ni > 0 {
                a = a.saturating_add(ni);
                if a > twotoi / 2 {
                    n = twotoi;
                    na = a;
                }
            }
            if a == *narray {
                break;
            }
            i = i.saturating_add(1);
            twotoi = twotoi.saturating_mul(2);
        }
        *narray = n;
        na
    }

    /// C++ `adjustasize`: grows a proposed array size over the keys that
    /// already follow it.
    fn adjustasize(&self, mut size: usize, ek: Option<Value>) -> usize {
        let tbound = !self.nodes.is_empty() || size < self.array.len();
        let ekindex = ek.and_then(array_index).unwrap_or(-1);
        while (size as i64).saturating_add(1) == ekindex
            || (tbound && !self.get_int((size as i64).saturating_add(1)).is_nil())
        {
            size = size.saturating_add(1);
        }
        size
    }

    /// The capacity in bytes `set(key, v)` would leave when it has to
    /// rehash, or `None` when it stores in place (or fails on the key).
    pub(crate) fn capacity_after_new_key(&self, key: Value) -> Option<usize> {
        if let Some(i) = array_index(key)
            && array_slot(i, self.array.len()).is_some()
        {
            return None;
        }
        match key {
            Value::Nil => return None,
            Value::Number(n) if n.is_nan() => return None,
            Value::Vector(v) if v.iter().any(|c| c.is_nan()) => return None,
            _ => {}
        }
        let key = normalize_key(key);
        if self.find_node(key).is_some() {
            return None;
        }
        let next_array = (self.array.len() as i64).saturating_add(1);
        if array_index(key) != Some(next_array) && self.has_free_node_for(key) {
            return None;
        }
        let (nasize, nhsize) = self.rehash_sizes(Some(key));
        Some(
            nasize
                .saturating_mul(16)
                .saturating_add(Self::node_bytes(nhsize)),
        )
    }

    /// Whether `insert_node(key, _)` would find room without a rehash.
    fn has_free_node_for(&self, key: Value) -> bool {
        if self.nodes.is_empty() {
            return false;
        }
        let mp = self.main_position(key);
        if self.nodes.get(mp).is_none_or(|n| n.val.is_nil()) {
            return true;
        }
        self.nodes
            .get(..self.lastfree)
            .is_some_and(|nodes| nodes.iter().rev().any(|n| n.key.is_nil()))
    }

    /// C++ `rehash`.
    fn rehash(&mut self, ek: Option<Value>) -> Result<(), TableError> {
        let (nasize, nh) = self.rehash_sizes(ek);
        self.resize(nasize, nh)
    }

    /// The array and hash sizes C++ `rehash` picks.
    fn rehash_sizes(&self, ek: Option<Value>) -> (usize, usize) {
        let mut nums = [0usize; MAXBITS + 1];
        let mut nasize = self.numusearray(&mut nums);
        let mut totaluse = nasize;
        for node in &self.nodes {
            if !node.key.is_nil() && !node.val.is_nil() {
                nasize = nasize.saturating_add(Self::countint(node.key, &mut nums));
                totaluse = totaluse.saturating_add(1);
            }
        }
        if let Some(ek) = ek {
            nasize = nasize.saturating_add(Self::countint(ek, &mut nums));
        }
        totaluse = totaluse.saturating_add(1);
        let na = Self::computesizes(&nums, &mut nasize);
        let mut nh = totaluse.saturating_sub(na);
        let nadjusted = self.adjustasize(nasize, ek);
        let aextra = nadjusted.saturating_sub(nasize);
        if aextra != 0 {
            nh = nh.saturating_sub(aextra);
            nasize = nadjusted.saturating_add(aextra);
            nasize = self.adjustasize(nasize, ek);
        }
        (nasize, nh)
    }

    /// C++ `resize`.
    fn resize(&mut self, nasize: usize, nhsize: usize) -> Result<(), TableError> {
        if nasize > MAXSIZE || nhsize > MAXSIZE {
            return Err(TableError::Overflow);
        }
        let old = std::mem::take(&mut self.nodes);
        self.set_sizenode(next_pow2(nhsize));
        if nasize > self.array.len() {
            self.array.resize(nasize, Value::Nil);
        }
        if nasize < self.array.len() {
            let tail: Vec<Value> = self.array.split_off(nasize);
            for (i, v) in tail.into_iter().enumerate() {
                if !v.is_nil() {
                    let key = Value::Number(nasize.saturating_add(i).saturating_add(1) as f64);
                    self.array_or_newkey(key, v)?;
                }
            }
        }
        for node in old.into_iter().rev() {
            if !node.key.is_nil() && !node.val.is_nil() {
                self.array_or_newkey(node.key, node.val)?;
            }
        }
        Ok(())
    }

    /// C++ `luaH_resizearray`, used by `SETLIST`, with the growth checked
    /// against `budget` first and settled after.
    pub(crate) fn resize_array(
        &mut self,
        nasize: usize,
        budget: &mut Budget,
    ) -> Result<(), TableError> {
        let asize = self.adjustasize(nasize, None);
        let hsize = self.nodes.len();
        let before = self.capacity_bytes();
        budget
            .check(Self::capacity_for(asize, hsize).saturating_sub(before))
            .map_err(|_| TableError::Memory)?;
        let result = self.resize(asize, hsize);
        budget.settle(before, self.capacity_bytes());
        result
    }

    /// A copy (DUPTABLE's template clone), with its capacity checked
    /// against and charged to `budget` first.
    pub(crate) fn try_clone(&self, budget: &mut Budget) -> Result<Table, TableError> {
        budget
            .charge(self.capacity_bytes())
            .map_err(|_| TableError::Memory)?;
        Ok(self.clone())
    }

    /// Removes the entries a collection cleared (C++ `cleartable`): an
    /// array slot whose value is cleared, a node whose key or value is.
    /// A removed node keeps its key as a dead key, as `set` to nil does.
    pub(crate) fn clear_entries(&mut self, mut cleared: impl FnMut(Value) -> bool) {
        for v in &mut self.array {
            if cleared(*v) {
                *v = Value::Nil;
            }
        }
        for node in &mut self.nodes {
            if !node.val.is_nil() && (cleared(node.key) || cleared(node.val)) {
                node.val = Value::Nil;
            }
        }
    }

    /// C++ `luaH_clear`: every entry is removed and both parts keep their
    /// capacity.
    pub(crate) fn clear(&mut self) {
        self.array.fill(Value::Nil);
        self.nodes.fill(Node::default());
        self.lastfree = self.nodes.len();
    }

    /// C++ `maxn`: the largest positive numeric key, scanning the array
    /// part and then the hash part.
    pub(crate) fn max_number_key(&self) -> f64 {
        let mut max = 0.0f64;
        for (i, v) in self.array.iter().enumerate() {
            if !v.is_nil() {
                max = i.saturating_add(1) as f64;
            }
        }
        for node in &self.nodes {
            if let Value::Number(k) = node.key
                && !node.val.is_nil()
                && k > max
            {
                max = k;
            }
        }
        max
    }

    /// Slots in both parts (C++ `sizearray + sizenode`, with the dummy node
    /// counted as one, as `table.move`'s sparse test sees it).
    pub(crate) fn slot_count(&self) -> usize {
        self.array.len().saturating_add(self.nodes.len().max(1))
    }

    /// C++ `luaH_getn` without the `aboundary` cache.
    pub(crate) fn len(&self) -> usize {
        let j = self.array.len();
        if j > 0
            && self
                .array
                .get(j.wrapping_sub(1))
                .is_some_and(|v| v.is_nil())
        {
            // Binary search for a border in the array part.
            let mut base = 0usize;
            let mut rest = j;
            loop {
                let half = rest >> 1;
                if half == 0 {
                    break;
                }
                let probe = base.saturating_add(half);
                if self.array.get(probe).is_some_and(|v| !v.is_nil()) {
                    base = probe;
                }
                rest = rest.saturating_sub(half);
            }
            usize::from(self.array.get(base).is_some_and(|v| !v.is_nil())).saturating_add(base)
        } else {
            // C++ asserts the array part is a border here: the rehash
            // trigger keeps `sizearray + 1` out of the hash part.
            j
        }
    }

    /// The traversal step shared by `next` and `FORGLOOP`: from cursor
    /// `index` (0 = start; array slots first, then nodes in slot order)
    /// find the next live entry, returning the cursor to resume from.
    #[inline]
    pub(crate) fn next_from(&self, mut index: usize) -> Option<(usize, Value, Value)> {
        let sizearray = self.array.len();
        while index < sizearray {
            if let Some(&v) = self.array.get(index)
                && !v.is_nil()
            {
                let next = index.saturating_add(1);
                return Some((next, Value::Number(next as f64), v));
            }
            index = index.saturating_add(1);
        }
        let mut node_index = index.saturating_sub(sizearray);
        while let Some(node) = self.nodes.get(node_index) {
            if !node.val.is_nil() {
                return Some((
                    sizearray.saturating_add(node_index).saturating_add(1),
                    node.key,
                    node.val,
                ));
            }
            node_index = node_index.saturating_add(1);
        }
        None
    }

    /// The cursor after `key` for `next` (C++ `luaH_next`'s index lookup).
    /// Dead keys are accepted. `None` means the key is not in the table.
    pub(crate) fn cursor_after(&self, key: Value) -> Option<usize> {
        if key.is_nil() {
            return Some(0);
        }
        if let Some(slot) = array_index(key).and_then(|i| array_slot(i, self.array.len())) {
            return Some(slot.saturating_add(1));
        }
        let node = self.find_node(normalize_key(key))?;
        Some(self.array.len().saturating_add(node).saturating_add(1))
    }

    /// Visits the values this table holds strongly, for the collector
    /// (C++ `traversetable`): array values unless values are weak, then
    /// each live node's key unless keys are weak and value unless values
    /// are weak. The metatable is the caller's. A dead key (a node whose
    /// value is nil) is not visited, as C++ `removeentry` makes it dead:
    /// it stays a valid `next` cursor by identity, and the object it named
    /// may be collected (memory safety never depends on liveness, heap.rs).
    pub(crate) fn trace(&self, weak_keys: bool, weak_values: bool, mut visit: impl FnMut(Value)) {
        if !weak_values {
            for &v in &self.array {
                visit(v);
            }
        }
        for node in &self.nodes {
            if !node.val.is_nil() {
                if !weak_keys {
                    visit(node.key);
                }
                if !weak_values {
                    visit(node.val);
                }
            }
        }
    }
}
