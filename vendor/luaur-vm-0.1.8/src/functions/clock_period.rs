pub(crate) fn clock_period() -> f64 {
    #[cfg(target_os = "windows")]
    {
        extern "system" {
            fn QueryPerformanceFrequency(lpFrequency: *mut i64) -> core::ffi::c_int;
        }
        let mut result: i64 = 0;
        unsafe {
            QueryPerformanceFrequency(&mut result);
        }
        1.0 / (result as f64)
    }
    #[cfg(target_vendor = "apple")]
    {
        #[repr(C)]
        struct mach_timebase_info_data_t {
            numer: u32,
            denom: u32,
        }
        extern "C" {
            fn mach_timebase_info(info: *mut mach_timebase_info_data_t) -> core::ffi::c_int;
        }
        let mut result = mach_timebase_info_data_t { numer: 0, denom: 0 };
        unsafe {
            mach_timebase_info(&mut result);
        }
        (result.numer as f64) / (result.denom as f64) * 1e-9
    }
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    {
        1e-9
    }
    #[cfg(target_arch = "wasm32")]
    {
        1e-3
    }
    #[cfg(not(any(
        target_os = "windows",
        target_vendor = "apple",
        target_os = "linux",
        target_os = "freebsd",
        target_arch = "wasm32"
    )))]
    {
        extern "C" {
            static CLOCKS_PER_SEC: core::ffi::c_long;
        }
        unsafe { 1.0 / (CLOCKS_PER_SEC as f64) }
    }
}
