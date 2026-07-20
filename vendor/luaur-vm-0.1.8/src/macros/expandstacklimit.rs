#[macro_export]
#[allow(non_snake_case)]
macro_rules! expandstacklimit {
    ($L:expr, $p:expr) => {
        unsafe {
            luaur_common::LUAU_ASSERT!(($p) <= (*$L).stack_last);
            if (*(*$L).ci).top < ($p) {
                (*(*$L).ci).top = ($p);
            }
        }
    };
}

pub use expandstacklimit;
