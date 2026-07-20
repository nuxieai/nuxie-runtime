use crate::macros::lua_tnone::LUA_TNONE;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! lua_isnone {
    ($L:expr, $n:expr) => {
        unsafe { $crate::functions::lua_type::lua_type($L, $n) } == $crate::macros::lua_tnone::LUA_TNONE
    };
}

pub use lua_isnone;
