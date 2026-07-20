use crate::functions::lua_l_typeerror_l::lua_l_typeerror_l;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaL_typeerror {
    ($L:expr, $narg:expr, $tname:expr) => {
        $crate::functions::lua_l_typeerror_l::lua_l_typeerror_l($L, $narg, $tname)
    };
}

pub use luaL_typeerror;
