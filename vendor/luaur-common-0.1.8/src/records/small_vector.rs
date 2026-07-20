//! Faithful hand-port of `Luau::SmallVector<T, N>`.
//!
//! Reference: `luau/Common/include/Luau/SmallVector.h` (whole file), Luau upstream.
//! A small-buffer-optimized vector: the first `N` elements live inline; on growth
//! it spills to a heap block of `1.5x`-or-`+4` capacity.
//!
//! **Sound-representation deviation (required, not cosmetic):** the C++ caches a
//! `ptr` that points either into its own inline `storage` or at the heap block.
//! A Rust value can be *moved* freely, which would dangle a pointer into inline
//! storage. So we do NOT cache a self-referential pointer: `heap` is null while
//! inline and the data pointer is recomputed from `storage`/`heap` on demand.
//! Behavior (SBO, capacities, element lifetimes) is otherwise identical.

extern crate alloc;

use alloc::alloc::{alloc, dealloc, handle_alloc_error, Layout};
use core::fmt;
use core::hash::{Hash, Hasher};
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::slice;

#[allow(non_camel_case_types)]
pub struct SmallVector<T, const N: usize> {
    /// Inline storage for the first `N` elements (small-buffer optimization).
    storage: [MaybeUninit<T>; N],
    /// Heap block once we outgrow `N`; null while inline. Never a pointer into
    /// `storage` — see the module note on move-safety.
    heap: *mut T,
    /// Number of live elements.
    count: u32,
    /// Capacity: `N` while inline, the heap block's capacity otherwise.
    max: u32,
}

impl<T, const N: usize> SmallVector<T, N> {
    pub fn new() -> Self {
        SmallVector {
            storage: [const { MaybeUninit::uninit() }; N],
            heap: ptr::null_mut(),
            count: 0,
            max: N as u32,
        }
    }

    fn is_heap(&self) -> bool {
        !self.heap.is_null()
    }

    /// Current data pointer, recomputed (never cached) so moves stay sound.
    fn data(&self) -> *const T {
        if self.heap.is_null() {
            self.storage.as_ptr() as *const T
        } else {
            self.heap
        }
    }

    fn data_mut(&mut self) -> *mut T {
        if self.heap.is_null() {
            self.storage.as_mut_ptr() as *mut T
        } else {
            self.heap
        }
    }

    pub fn as_slice(&self) -> &[T] {
        // SAFETY: `data()` is valid for `count` initialized, contiguous elements.
        unsafe { slice::from_raw_parts(self.data(), self.count as usize) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let len = self.count as usize;
        let ptr = self.data_mut();
        // SAFETY: as `as_slice`, with unique access from `&mut self`.
        unsafe { slice::from_raw_parts_mut(ptr, len) }
    }

    pub fn size(&self) -> u32 {
        self.count
    }

    pub fn capacity(&self) -> u32 {
        self.max
    }

    pub fn empty(&self) -> bool {
        self.count == 0
    }

    pub fn front(&self) -> &T {
        assert!(self.count > 0);
        &self.as_slice()[0]
    }

    pub fn back(&self) -> &T {
        assert!(self.count > 0);
        &self.as_slice()[self.count as usize - 1]
    }

    /// std-style alias for `push_back` — translations use Vec idioms.
    pub fn push(&mut self, value: T) {
        self.push_back(value);
    }

    pub fn push_back(&mut self, value: T) {
        if self.count == self.max {
            self.grow(self.count + 1);
        }
        let index = self.count as usize;
        // SAFETY: we just ensured capacity for one more element at `index`.
        unsafe { self.data_mut().add(index).write(value) };
        self.count += 1;
    }

    /// `emplace_back` collapses to `push_back` of the constructed value; Rust has
    /// no in-place variadic construction, and the move is free.
    pub fn emplace_back(&mut self, value: T) -> &mut T {
        self.push_back(value);
        let index = self.count as usize - 1;
        &mut self.as_mut_slice()[index]
    }

    pub fn pop_back(&mut self) {
        assert!(self.count > 0);
        self.count -= 1;
        let index = self.count as usize;
        // SAFETY: element at `index` was live and is now logically removed.
        unsafe { ptr::drop_in_place(self.data_mut().add(index)) };
    }

    pub fn clear(&mut self) {
        while self.count > 0 {
            self.pop_back();
        }
    }

    pub fn reserve(&mut self, reserve_size: u32) {
        if reserve_size > self.max {
            self.grow(reserve_size);
        }
    }

    /// Faithful to the C++ growth policy: `1.5x`, or `+4` when that is too small.
    fn grow(&mut self, new_size: u32) {
        let new_size = if self.max + (self.max >> 1) > new_size {
            self.max + (self.max >> 1)
        } else {
            new_size + 4
        };
        assert!(new_size < 0x4000_0000);

        let layout = Layout::array::<T>(new_size as usize).expect("SmallVector capacity overflow");
        // SAFETY: `Layout::array` rejects zero-size layouts for non-ZST `T`; null
        // is handled immediately below.
        let new_data = unsafe { alloc(layout) as *mut T };
        if new_data.is_null() {
            handle_alloc_error(layout);
        }

        // SAFETY: move (bitwise) the `count` live elements into the new block. The
        // sources are then logically uninitialized and must NOT be dropped, which
        // is exactly what switching `heap`/`is_heap` to the new block achieves.
        unsafe {
            ptr::copy_nonoverlapping(self.data(), new_data, self.count as usize);
            if self.is_heap() {
                let old_layout =
                    Layout::array::<T>(self.max as usize).expect("SmallVector capacity overflow");
                dealloc(self.heap as *mut u8, old_layout);
            }
        }

        self.heap = new_data;
        self.max = new_size;
    }
}

impl<T: Default, const N: usize> SmallVector<T, N> {
    pub fn resize(&mut self, new_size: u32) {
        if new_size > self.count {
            if new_size > self.max {
                self.grow(new_size);
            }
            for index in self.count as usize..new_size as usize {
                // SAFETY: capacity ensured above; writing a fresh element.
                unsafe { self.data_mut().add(index).write(T::default()) };
            }
        } else {
            while self.count > new_size {
                self.pop_back();
            }
        }
        self.count = new_size;
    }
}

impl<T, const N: usize> Default for SmallVector<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Deref for SmallVector<T, N> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const N: usize> DerefMut for SmallVector<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T: Clone, const N: usize> Clone for SmallVector<T, N> {
    fn clone(&self) -> Self {
        let mut copy = Self::new();
        copy.reserve(self.count);
        for value in self.as_slice() {
            copy.push_back(value.clone());
        }
        copy
    }
}

impl<T, const N: usize> Drop for SmallVector<T, N> {
    fn drop(&mut self) {
        self.clear();
        if self.is_heap() {
            let layout =
                Layout::array::<T>(self.max as usize).expect("SmallVector capacity overflow");
            // SAFETY: `heap` was allocated with this layout in `grow`.
            unsafe { dealloc(self.heap as *mut u8, layout) };
        }
    }
}

impl<T: PartialEq, const N: usize> PartialEq for SmallVector<T, N> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Eq, const N: usize> Eq for SmallVector<T, N> {}

impl<T: Hash, const N: usize> Hash for SmallVector<T, N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl<T: fmt::Debug, const N: usize> fmt::Debug for SmallVector<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a SmallVector<T, N> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl<T, const N: usize> FromIterator<T> for SmallVector<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut out = Self::new();
        for value in iter {
            out.push_back(value);
        }
        out
    }
}

// SAFETY: a `SmallVector<T>` owns its `T`s (inline or on the heap); the raw
// `heap` pointer carries no shared state, so it is `Send`/`Sync` exactly when `T`
// is, matching `alloc::vec::Vec`.
unsafe impl<T: Send, const N: usize> Send for SmallVector<T, N> {}
unsafe impl<T: Sync, const N: usize> Sync for SmallVector<T, N> {}

impl<T, const N: usize> crate::records::dense_hash_table::DenseDefault for SmallVector<T, N> {
    fn dense_default() -> Self {
        Self::new()
    }
}
