pub(crate) fn get_clock_period() -> f64 {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::System::Performance::QueryPerformanceFrequency;
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
            fn mach_timebase_info(info: *mut mach_timebase_info_data_t) -> i32;
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
        // `wasm32-unknown-unknown` exposes no clock source (no libc, no WASI),
        // so TimeTrace profiling is a no-op. A 1 ns period keeps the
        // `period * timestamp` math finite alongside the frozen timestamp below.
        1e-9
    }
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(any(
            target_os = "windows",
            target_vendor = "apple",
            target_os = "linux",
            target_os = "freebsd"
        ))
    ))]
    {
        // Fallback to libc CLOCKS_PER_SEC
        // EXTERNAL_CRATE_REQUIRED: libc - access to CLOCKS_PER_SEC
        1.0 / (libc::CLOCKS_PER_SEC as f64)
    }
}
