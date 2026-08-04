#[allow(non_upper_case_globals)]
#[macro_export]
macro_rules! luaR_checkoffsetinbounds {
    ($inst:expr, $offset:expr) => {
        $offset < unsafe { (*(*$inst).lclass).numberofallmembers }
    };
}

pub use luaR_checkoffsetinbounds;
