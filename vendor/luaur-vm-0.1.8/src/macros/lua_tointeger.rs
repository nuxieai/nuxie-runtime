use crate::functions::lua_tointegerx::lua_tointegerx;

#[allow(non_upper_case_globals)]
pub const LUA_TOINTEGER: () = ();

#[allow(non_snake_case)]
#[macro_export]
macro_rules! lua_tointeger {
    ($l:expr, $i:expr) => {
        unsafe { $crate::functions::lua_tointegerx::lua_tointegerx($l, $i, core::ptr::null_mut()) }
    };
}

pub use lua_tointeger;
