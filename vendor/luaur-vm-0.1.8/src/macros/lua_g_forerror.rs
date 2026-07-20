use crate::functions::lua_g_forerror_l::lua_g_forerror_l;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaG_forerror {
    ($l:expr, $o:expr, $what:expr) => {{
        $crate::functions::lua_g_forerror_l::lua_g_forerror_l($l, $o, $what);
    }};
}

pub use luaG_forerror;
