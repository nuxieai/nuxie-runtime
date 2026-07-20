use crate::functions::lua_pushfstring_l::lua_pushfstring_l;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! lua_pushfstring {
    ($l:expr, $fmt:expr $(, $($arg:expr),* )? ) => {{
        $crate::functions::lua_pushfstring_l::lua_pushfstring_l($l, $fmt $(, $($arg),* )?);
    }};
}

pub use lua_pushfstring;
