//! Translation of renderer/include/rive/renderer/scoped_autorelease_pool.hpp.

/// Drains autoreleased objects at the end of a scope, including on threads
/// without a run loop. Off Apple platforms the scope is a no-op.
///
/// The closure is Rust's spelling of the noncopyable upstream RAII scope:
/// nested pools cannot be dropped out of order or moved to another thread.
pub struct ScopedAutoreleasePool {
    _private: (),
}

#[cfg(all(test, target_vendor = "apple"))]
#[path = "scoped_autorelease_pool_test.rs"]
mod tests;

impl ScopedAutoreleasePool {
    #[cfg(target_vendor = "apple")]
    pub fn with<T, F>(body: F) -> T
    where
        F: FnOnce() -> T + objc2::rc::AutoreleaseSafe,
    {
        objc2::rc::autoreleasepool(|_| body())
    }

    #[cfg(not(target_vendor = "apple"))]
    pub fn with<T>(body: impl FnOnce() -> T) -> T {
        body()
    }
}
