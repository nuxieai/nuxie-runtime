use crate::functions::lua_l_optlstring::lua_l_optlstring;

#[allow(non_upper_case_globals)]
pub const LUA_L_OPTSTRING: () = ();

#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaL_optstring {
    ($l:expr, $n:expr, $d:expr) => {{
        let mut len: usize = 0;
        unsafe {
            $crate::functions::lua_l_optlstring::lua_l_optlstring(
                $l,
                $n,
                $d,
                &mut len as *mut usize,
            )
        }
    }};
}

pub use luaL_optstring;
