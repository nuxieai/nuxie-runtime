#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaL_argexpected {
    ($L:expr, $cond:expr, $arg:expr, $tname:expr) => {
        if !($cond) {
            $crate::luaL_typeerror!($L, $arg, $tname);
        }
    };
}

pub use luaL_argexpected;
