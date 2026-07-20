#[allow(non_upper_case_globals)]
#[macro_export]
macro_rules! luaR_checkoffsetinbounds {
    ($inst:expr, $offset:expr) => {
        (core::ffi::c_int::from($offset) >= 0)
            && (core::ffi::c_int::from($offset) < unsafe { (*(*$inst).lclass).numberofallmembers })
    };
}

pub use luaR_checkoffsetinbounds;
