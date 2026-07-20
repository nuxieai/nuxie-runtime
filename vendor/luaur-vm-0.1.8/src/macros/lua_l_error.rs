use crate::functions::lua_l_error_l::lua_l_error_l;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaL_error {
    ($l:expr, $fmt:expr $(, $($arg:expr),+ )? $(,)? ) => {{
        unsafe { $crate::functions::lua_l_error_l::lua_l_error_l(
            $l,
            core::ptr::null(),
            core::format_args!($fmt $(, $($arg),* )?),
        ) };
    }};
}

pub use luaL_error;
