pub(crate) fn clock_timestamp() -> f64 {
    #[cfg(windows)]
    {
        use core::mem::MaybeUninit;
        #[link(name = "kernel32")]
        extern "system" {
            fn QueryPerformanceCounter(lpPerformanceCount: *mut i64) -> i32;
        }
        let mut result = 0i64;
        unsafe {
            QueryPerformanceCounter(&mut result);
        }
        result as f64
    }
    #[cfg(target_vendor = "apple")]
    {
        extern "C" {
            fn mach_absolute_time() -> u64;
        }
        unsafe { mach_absolute_time() as f64 }
    }
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        use core::mem::MaybeUninit;
        #[repr(C)]
        struct timespec {
            tv_sec: core::ffi::c_long,
            tv_nsec: core::ffi::c_long,
        }
        extern "C" {
            fn clock_gettime(clk_id: i32, tp: *mut timespec) -> i32;
        }
        const CLOCK_MONOTONIC: i32 = 1;
        let mut now = MaybeUninit::<timespec>::uninit();
        unsafe {
            clock_gettime(CLOCK_MONOTONIC, now.as_mut_ptr());
            let now = now.assume_init();
            (now.tv_sec as f64) * 1e9 + (now.tv_nsec as f64)
        }
    }
    #[cfg(target_os = "emscripten")]
    {
        extern "C" {
            fn emscripten_get_now() -> f64;
        }
        unsafe { emscripten_get_now() }
    }
    #[cfg(not(any(
        windows,
        target_vendor = "apple",
        target_os = "linux",
        target_os = "freebsd",
        target_os = "emscripten"
    )))]
    {
        extern "C" {
            fn clock() -> core::ffi::c_long;
        }
        unsafe { clock() as f64 }
    }
}
