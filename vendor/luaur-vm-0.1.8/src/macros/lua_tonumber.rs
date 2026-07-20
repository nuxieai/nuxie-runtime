use crate::functions::lua_tonumberx::lua_tonumberx;

#[allow(non_upper_case_globals)]
pub const LUA_TONUMBER: () = ();

#[allow(non_snake_case)]
#[macro_export]
macro_rules! lua_tonumber {
    ($l:expr, $i:expr) => {
        unsafe { $crate::functions::lua_tonumberx::lua_tonumberx($l, $i, core::ptr::null_mut()) }
    };
}

pub use lua_tonumber;
