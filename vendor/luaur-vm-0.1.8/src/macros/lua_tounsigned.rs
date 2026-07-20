use crate::functions::lua_tounsignedx::lua_tounsignedx;

#[allow(non_upper_case_globals)]
pub const LUA_TOUNSIGNED: () = ();

#[allow(non_snake_case)]
#[macro_export]
macro_rules! lua_tounsigned {
    ($l:expr, $i:expr) => {
        unsafe {
            $crate::functions::lua_tounsignedx::lua_tounsignedx($l, $i, core::ptr::null_mut())
        }
    };
}

pub use lua_tounsigned;
