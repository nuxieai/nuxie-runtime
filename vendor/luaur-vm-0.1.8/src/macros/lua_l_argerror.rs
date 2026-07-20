#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaL_argerror {
    ($L:expr, $narg:expr, $extramsg:expr) => {
        $crate::functions::lua_l_argerror_l::luaL_argerrorL($L, $narg, $extramsg)
    };
}

pub use luaL_argerror;
