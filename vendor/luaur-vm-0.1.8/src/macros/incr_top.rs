#[macro_export]
#[allow(non_snake_case)]
macro_rules! incr_top {
    ($L:expr) => {{
        crate::macros::lua_d_checkstack::luaD_checkstack!($L, 1);
        unsafe {
            (*$L).top = (*$L).top.add(1);
        }
    }};
}

pub use incr_top;
