#[allow(non_snake_case)]
macro_rules! luaD_checkstack {
    ($L:expr, $n:expr) => {
        if crate::macros::stacklimitreached::stacklimitreached($L, $n) {
            $crate::functions::lua_d_growstack::lua_d_growstack($L, $n);
        } else {
            crate::macros::condhardstacktests::condhardstacktests!(
                crate::functions::lua_d_reallocstack::luaD_reallocstack(
                    $L,
                    $L.stacksize - crate::macros::extra_stack::EXTRA_STACK,
                    0
                )
            );
        }
    };
}

pub(crate) use luaD_checkstack;
