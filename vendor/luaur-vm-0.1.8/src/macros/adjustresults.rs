#[macro_export]
#[allow(non_snake_case)]
macro_rules! adjustresults {
    ($L:expr, $nres:expr) => {
        if $nres == crate::macros::lua_multret::LUA_MULTRET
            && unsafe { (*$L).top.offset_from((*(*$L).ci).top) >= 0 }
        {
            unsafe { (*(*$L).ci).top = (*$L).top };
        }
    };
}

pub use adjustresults;
