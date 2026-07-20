#[allow(non_camel_case_types)]
pub type time_t = i64;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
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
}

#[allow(non_snake_case)]
pub fn gmtime_r(timep: *const time_t, result: *mut tm) -> *mut tm {
    unsafe {
        #[cfg(target_os = "windows")]
        {
            extern "C" {
                // `gmtime_s` is an inline wrapper in MSVC's <time.h>; the real
                // exported UCRT symbol is `_gmtime64_s` (__time64_t = i64).
                fn _gmtime64_s(result: *mut tm, timep: *const time_t) -> core::ffi::c_int;
            }
            if _gmtime64_s(result, timep) == 0 {
                result
            } else {
                core::ptr::null_mut()
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            extern "C" {
                fn gmtime_r(timep: *const time_t, result: *mut tm) -> *mut tm;
            }
            gmtime_r(timep, result)
        }
    }
}
