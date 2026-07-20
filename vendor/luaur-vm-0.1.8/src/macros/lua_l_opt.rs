use crate::macros::lua_isnoneornil::lua_isnoneornil;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaL_opt {
    ($L:expr, $f:expr, $n:expr, $d:expr) => {
        if $crate::macros::lua_isnoneornil::lua_isnoneornil!($L, $n) {
            $d
        } else {
            $f($L, $n)
        }
    };
}

pub use luaL_opt;
