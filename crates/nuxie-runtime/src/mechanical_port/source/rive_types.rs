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
pub unsafe fn inline_memcpy(destination: *mut u8, source: *const u8, count: usize) {
    unsafe { std::ptr::copy_nonoverlapping(source, destination, count) };
}
