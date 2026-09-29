//! Upstream tests/unit_tests/renderer/scoped_autorelease_pool_test.cpp.

use super::ScopedAutoreleasePool;
use objc2_core_foundation::{CFMutableString, CFRetained};
use std::ptr::NonNull;

fn retain_autorelease(object: &CFMutableString) {
    // SAFETY: The caller supplies a live object. Retain creates one additional
    // owning reference, transferred to the current autorelease pool below.
    let retained = unsafe { CFRetained::retain(NonNull::from(object)) };
    CFRetained::autorelease_ptr(retained);
}

#[test]
fn pool_drains_autoreleases_on_a_bare_thread() {
    // Mutable strings are heap objects, never tagged pointers, so their retain
    // count is real. None is upstream's kCFAllocatorDefault.
    let object = CFMutableString::new(None, 0).expect("heap mutable string");
    assert_eq!(object.retain_count(), 1);
    let address = CFRetained::as_ptr(&object).as_ptr() as usize;
    std::thread::spawn(move || {
        // SAFETY: The parent retains the object and joins before releasing it.
        // Neither thread mutates its string contents; the parent only inspects
        // its count before spawn and after join. Only CF retain/autorelease
        // operations and count inspection occur here. This narrowly mirrors
        // upstream's raw CFMutableStringRef capture without marking CF strings
        // generally Send or Sync.
        let object = unsafe { &*(address as *const CFMutableString) };
        ScopedAutoreleasePool::with(|| {
            retain_autorelease(object);
            assert_eq!(object.retain_count(), 2);
        });
        assert_eq!(object.retain_count(), 1);
    })
    .join()
    .expect("bare thread assertions");
    assert_eq!(object.retain_count(), 1);
    drop(object); // CFRelease of the original owning reference.
}

#[test]
fn pools_nest() {
    let object = CFMutableString::new(None, 0).expect("heap mutable string");
    let address = CFRetained::as_ptr(&object).as_ptr() as usize;
    std::thread::spawn(move || {
        // SAFETY: As above, the original owning reference outlives the joined
        // thread; no contents are mutated and no concurrent access occurs.
        let object = unsafe { &*(address as *const CFMutableString) };
        ScopedAutoreleasePool::with(|| {
            retain_autorelease(object);
            ScopedAutoreleasePool::with(|| {
                retain_autorelease(object);
                assert_eq!(object.retain_count(), 3);
            });
            assert_eq!(object.retain_count(), 2);
        });
    })
    .join()
    .expect("nested pool assertions");
    assert_eq!(object.retain_count(), 1);
    drop(object);
}
