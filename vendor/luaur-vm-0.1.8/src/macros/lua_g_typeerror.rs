use crate::functions::lua_g_typeerror_l::lua_g_typeerror_l;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaG_typeerror {
    ($l:expr, $o:expr, $opname:expr) => {{
        $crate::functions::lua_g_typeerror_l::lua_g_typeerror_l($l, $o, $opname);
    }};
}

pub use luaG_typeerror;
