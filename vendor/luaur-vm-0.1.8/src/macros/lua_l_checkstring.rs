use crate::functions::lua_l_checklstring::lua_l_checklstring;

#[allow(non_upper_case_globals)]
pub const LUA_L_CHECKSTRING: () = ();

#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaL_checkstring {
    ($l:expr, $n:expr) => {{
        let mut len: usize = 0;
        unsafe {
            $crate::functions::lua_l_checklstring::lua_l_checklstring(
                $l,
                $n,
                &mut len as *mut usize,
            )
        }
    }};
}

pub use luaL_checkstring;
