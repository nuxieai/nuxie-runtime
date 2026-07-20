#[macro_export]
#[allow(non_snake_case)]
macro_rules! checkresults {
    ($L:expr, $na:expr, $nr:expr) => {
        crate::macros::api_check::api_check!(
            $L,
            ($nr) == crate::macros::LUA_MULTRET
                || (unsafe { (*(*$L).ci).top.offset_from((*$L).top) } >= (($nr) - ($na)) as isize)
        );
    };
}

pub use checkresults;
