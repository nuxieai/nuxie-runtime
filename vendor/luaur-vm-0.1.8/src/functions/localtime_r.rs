#[allow(non_camel_case_types)]
pub type time_t = i64;

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Default)]
pub struct tm {
    pub tm_sec: core::ffi::c_int,
    pub tm_min: core::ffi::c_int,
    pub tm_hour: core::ffi::c_int,
    pub tm_mday: core::ffi::c_int,
    pub tm_mon: core::ffi::c_int,
    pub tm_year: core::ffi::c_int,
    pub tm_wday: core::ffi::c_int,
    pub tm_yday: core::ffi::c_int,
    pub tm_isdst: core::ffi::c_int,
    #[cfg(not(target_os = "windows"))]
    pub tm_gmtoff: core::ffi::c_long,
    #[cfg(not(target_os = "windows"))]
    pub tm_zone: *const core::ffi::c_char,
}

#[allow(non_snake_case)]
pub unsafe fn localtime_r(timep: *const time_t, result: *mut tm) -> *mut tm {
    #[cfg(target_os = "windows")]
    {
        extern "C" {
            // MSVC's `localtime_s` is an inline wrapper in <time.h>, so it has no
            // exported symbol to link against ("unresolved external symbol
            // localtime_s"). The real UCRT export is `_localtime64_s`, taking a
            // `__time64_t` (our `time_t = i64`).
            fn _localtime64_s(result: *mut tm, timep: *const time_t) -> core::ffi::c_int;
        }
        if _localtime64_s(result, timep) == 0 {
            result
        } else {
            core::ptr::null_mut()
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        extern "C" {
            fn localtime_r(timep: *const time_t, result: *mut tm) -> *mut tm;
        }
        localtime_r(timep, result)
    }
}
