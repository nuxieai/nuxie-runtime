use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
    ptr::NonNull,
    sync::atomic::{AtomicI32, Ordering},
};

pub struct RefCnt {
    count: AtomicI32,
}

impl Default for RefCnt {
    fn default() -> Self {
        Self::new()
    }
}

impl RefCnt {
    pub const fn new() -> Self {
        Self {
            count: AtomicI32::new(1),
        }
    }

    pub fn add_ref(&self) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn remove_ref(&self) -> bool {
        self.count.fetch_add(-1, Ordering::AcqRel) == 1
    }

    pub fn debugging_refcnt(&self) -> i32 {
        self.count.load(Ordering::Relaxed)
    }
}

pub unsafe trait RefCounted: Sized {
    fn ref_count(&self) -> &RefCnt;

    unsafe fn on_ref_count_reached_zero(pointer: NonNull<Self>) {
        unsafe { drop(Box::from_raw(pointer.as_ptr())) };
    }
}

pub unsafe fn safe_ref<T: RefCounted>(pointer: Option<NonNull<T>>) -> Option<NonNull<T>> {
    if let Some(pointer) = pointer {
        unsafe { pointer.as_ref() }.ref_count().add_ref();
    }
    pointer
}

pub unsafe fn safe_unref<T: RefCounted>(pointer: Option<NonNull<T>>) {
    if let Some(pointer) = pointer
        && unsafe { pointer.as_ref() }.ref_count().remove_ref()
    {
        unsafe { T::on_ref_count_reached_zero(pointer) };
    }
}

pub struct Rcp<T: RefCounted> {
    pointer: Option<NonNull<T>>,
    marker: PhantomData<T>,
}

impl<T: RefCounted> Default for Rcp<T> {
    fn default() -> Self {
        Self {
            pointer: None,
            marker: PhantomData,
        }
    }
}

impl<T: RefCounted> Rcp<T> {
    pub unsafe fn from_raw(pointer: *mut T) -> Self {
        Self {
            pointer: NonNull::new(pointer),
            marker: PhantomData,
        }
    }

    pub fn get(&self) -> *mut T {
        self.pointer.map_or(std::ptr::null_mut(), NonNull::as_ptr)
    }

    pub fn reset(&mut self, pointer: Option<NonNull<T>>) {
        let old_pointer = self.pointer;
        self.pointer = pointer;
        unsafe { safe_unref(old_pointer) };
    }

    pub fn release(&mut self) -> *mut T {
        self.pointer
            .take()
            .map_or(std::ptr::null_mut(), NonNull::as_ptr)
    }

    pub fn swap(&mut self, other: &mut Self) {
        std::mem::swap(&mut self.pointer, &mut other.pointer);
    }
}

impl<T: RefCounted> Clone for Rcp<T> {
    fn clone(&self) -> Self {
        Self {
            pointer: unsafe { safe_ref(self.pointer) },
            marker: PhantomData,
        }
    }
}

impl<T: RefCounted> Drop for Rcp<T> {
    fn drop(&mut self) {
        unsafe { safe_unref(self.pointer) };
    }
}

impl<T: RefCounted> std::ops::Deref for Rcp<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            self.pointer
                .expect("cannot dereference a null rcp")
                .as_ref()
        }
    }
}

impl<T: RefCounted> std::ops::DerefMut for Rcp<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            self.pointer
                .as_mut()
                .expect("cannot dereference a null rcp")
                .as_mut()
        }
    }
}

impl<T: RefCounted, U: RefCounted> PartialEq<Rcp<U>> for Rcp<T> {
    fn eq(&self, other: &Rcp<U>) -> bool {
        self.get().cast::<()>() == other.get().cast::<()>()
    }
}

impl<T: RefCounted> Eq for Rcp<T> {}

impl<T: RefCounted> Hash for Rcp<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pointer.hash(state);
    }
}

pub fn make_rcp<T: RefCounted>(value: T) -> Rcp<T> {
    let pointer = NonNull::from(Box::leak(Box::new(value)));
    Rcp {
        pointer: Some(pointer),
        marker: PhantomData,
    }
}

pub unsafe fn ref_rcp<T: RefCounted>(pointer: *mut T) -> Rcp<T> {
    let pointer = NonNull::new(pointer);
    let pointer = unsafe { safe_ref(pointer) };
    Rcp {
        pointer,
        marker: PhantomData,
    }
}

pub unsafe fn static_rcp_cast<U: RefCounted, T: RefCounted>(mut pointer: Rcp<T>) -> Rcp<U> {
    unsafe { Rcp::from_raw(pointer.release().cast::<U>()) }
}
