//! Correspondence owner for pinned `rive/refcnt.hpp`.
//!
//! The C++ owner supplies intrusive `RefCnt`/`rcp` lifetime management. The
//! Rust port deliberately exposes no equivalent raw-pointer API: each live
//! owner selects its truthful native representation explicitly (`Box` for a
//! unique renderer resource, `Rc` for shared immutable identity, or
//! `Rc<RefCell<_>>` for shared single-threaded mutable identity). This keeps
//! allocation, cloning, mutation, and destruction structural and prevents an
//! intrusive compatibility layer from manufacturing aliasing references.

use std::{rc::Rc, sync::Arc};

/// Observable shared-ownership operations supplied by C++ `RefCnt`/`rcp`.
///
/// Live translated owners choose `Rc` or `Arc` at their ownership boundary;
/// this trait records the common behavior without reintroducing intrusive raw
/// pointers or a second smart-pointer implementation.
pub trait NativeRefCount: Clone {
    fn debugging_refcnt(&self) -> usize;
    fn ptr_eq(&self, other: &Self) -> bool;
}

impl<T: ?Sized> NativeRefCount for Rc<T> {
    fn debugging_refcnt(&self) -> usize {
        Rc::strong_count(self)
    }

    fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(self, other)
    }
}

impl<T: ?Sized> NativeRefCount for Arc<T> {
    fn debugging_refcnt(&self) -> usize {
        Arc::strong_count(self)
    }

    fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(self, other)
    }
}

/// Safe-Rust translation of nullable `safe_ref`: clone an existing owner.
pub fn safe_ref<T: Clone>(value: Option<&T>) -> Option<T> {
    value.cloned()
}

/// Safe-Rust translation of nullable `safe_unref`: release one owner.
pub fn safe_unref<T>(value: Option<T>) {
    drop(value);
}

/// Safe-Rust translation of `rcp::release` for an optional owner slot.
pub fn release<T>(value: &mut Option<T>) -> Option<T> {
    value.take()
}

/// Safe-Rust translation of `rcp::reset`.
pub fn reset<T>(value: &mut Option<T>, replacement: Option<T>) {
    *value = replacement;
}

/// Safe-Rust translation of `rcp::swap`.
pub fn swap<T>(left: &mut T, right: &mut T) {
    std::mem::swap(left, right);
}
