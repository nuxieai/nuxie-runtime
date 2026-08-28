pub const BUILD_FOR_APPLE: bool = cfg!(target_vendor = "apple");
pub const BUILD_FOR_IOS: bool = cfg!(target_os = "ios");
pub const BUILD_FOR_OSX: bool = cfg!(target_os = "macos");
pub const NO_STD_SYSTEM: bool = BUILD_FOR_APPLE;
pub const DEBUG: bool = cfg!(debug_assertions);
pub const RELEASE: bool = !DEBUG;

#[cold]
#[inline(never)]
pub fn unreachable_reached() -> ! {
    debug_assert!(false, "unreachable reached");
    unreachable!("unreachable reached")
}

#[inline(always)]
/// Copy `count` non-overlapping bytes across a call-scoped FFI boundary.
///
/// # Safety
///
/// `source` must be readable and `destination` writable for `count` bytes;
/// both ranges must be live, properly aligned for bytes, and non-overlapping.
/// Neither pointer is retained.
pub unsafe fn inline_memcpy(destination: *mut u8, source: *const u8, count: usize) {
    // SAFETY: the caller supplies the valid non-overlapping ranges documented
    // by this function's contract.
    unsafe { std::ptr::copy_nonoverlapping(source, destination, count) };
}
