//! renderer/include/rive/renderer/next_ptr_range.hpp at 51b35dc1.
use core::{marker::PhantomData, ptr::NonNull};

pub trait NextPtrNode: Sized {
    fn next_ptr(&self) -> Option<NonNull<Self>>;
}
pub trait NextPtrNodeMut: NextPtrNode {
    fn set_next_ptr(&mut self, next: Option<NonNull<Self>>);
}

#[derive(Debug)]
pub struct NextPtrIterator<'a, T: NextPtrNode> {
    node: *const T,
    lifetime: PhantomData<&'a T>,
}
impl<'a, T: NextPtrNode> Copy for NextPtrIterator<'a, T> {}
impl<'a, T: NextPtrNode> Clone for NextPtrIterator<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a, T: NextPtrNode> NextPtrIterator<'a, T> {
    /// Nodes must remain live and immutable for 'a; their next links must be valid.
    pub unsafe fn new(node: *const T) -> Self {
        Self {
            node,
            lifetime: PhantomData,
        }
    }
    pub fn value(&self) -> &'a T {
        assert!(!self.node.is_null());
        unsafe { &*self.node }
    }
    pub fn as_ptr(&self) -> *const T {
        self.node
    }
    pub fn advance(&mut self) {
        self.node = self
            .value()
            .next_ptr()
            .map_or(core::ptr::null(), |p| p.as_ptr());
    }
}
impl<T: NextPtrNode> PartialEq for NextPtrIterator<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}
impl<T: NextPtrNode> Eq for NextPtrIterator<'_, T> {}

pub struct NextPtrRange<'a, T: NextPtrNode> {
    first: *const T,
    end: *const T,
    lifetime: PhantomData<&'a T>,
}
impl<'a, T: NextPtrNode> NextPtrRange<'a, T> {
    /// The immutable linked nodes must be live for 'a, and end must be reachable (or null).
    pub unsafe fn new(first: *const T, end: *const T) -> Self {
        Self {
            first,
            end,
            lifetime: PhantomData,
        }
    }
    pub fn begin(&self) -> NextPtrIterator<'a, T> {
        unsafe { NextPtrIterator::new(self.first) }
    }
    pub fn end(&self) -> NextPtrIterator<'a, T> {
        unsafe { NextPtrIterator::new(self.end) }
    }
}
pub struct NextPtrRangeIterator<'a, T: NextPtrNode> {
    current: NextPtrIterator<'a, T>,
    end: NextPtrIterator<'a, T>,
}
impl<'a, T: NextPtrNode> Iterator for NextPtrRangeIterator<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            return None;
        }
        let node = self.current.value();
        self.current.advance();
        Some(node)
    }
}
impl<'a, T: NextPtrNode> IntoIterator for NextPtrRange<'a, T> {
    type Item = &'a T;
    type IntoIter = NextPtrRangeIterator<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        NextPtrRangeIterator {
            current: self.begin(),
            end: self.end(),
        }
    }
}

/// Exclusive counterpart for the source's mutable T* range.
pub struct NextPtrRangeMut<'a, T: NextPtrNode> {
    current: *mut T,
    end: *mut T,
    lifetime: PhantomData<&'a mut T>,
}
impl<'a, T: NextPtrNode> NextPtrRangeMut<'a, T> {
    /// Every node before end must be distinct, valid, and exclusively borrowed for 'a.
    pub unsafe fn new(first: *mut T, end: *mut T) -> Self {
        Self {
            current: first,
            end,
            lifetime: PhantomData,
        }
    }
}
impl<'a, T: NextPtrNode> Iterator for NextPtrRangeMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            return None;
        }
        let node = unsafe { &mut *self.current };
        self.current = node
            .next_ptr()
            .map_or(core::ptr::null_mut(), |p| p.as_ptr());
        Some(node)
    }
}
