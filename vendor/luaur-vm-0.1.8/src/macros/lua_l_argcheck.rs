#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaL_argcheck {
    ($L:expr, $cond:expr, $arg:expr, $extramsg:expr) => {
        if !($cond) {
            $crate::macros::lua_l_argerror::luaL_argerror!($L, $arg, $extramsg);
        }
    };
}

pub use luaL_argcheck;
