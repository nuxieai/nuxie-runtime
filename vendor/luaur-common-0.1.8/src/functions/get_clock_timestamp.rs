pub(crate) fn get_clock_timestamp() -> f64 {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::System::Performance::QueryPerformanceCounter;
        let mut result: i64 = 0;
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
        let mut now = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        unsafe {
            libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut now);
        }
        (now.tv_sec as f64) * 1e9 + (now.tv_nsec as f64)
    }
    #[cfg(target_arch = "wasm32")]
    {
        // `wasm32-unknown-unknown` has no clock source; the profiler clock is
        // frozen at 0. See `get_clock_period` for the matching no-op period.
        0.0
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
        unsafe { libc::clock() as f64 }
    }
}
