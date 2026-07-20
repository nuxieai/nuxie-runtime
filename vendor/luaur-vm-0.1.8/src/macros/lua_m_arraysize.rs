use crate::functions::lua_m_toobig::lua_m_toobig;
use crate::macros::cast_to::cast_to;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! lua_m_arraysize {
    ($l:expr, $n:expr, $e:expr) => {
        if $crate::macros::cast_to::cast_to!(usize, $n) <= usize::MAX / $crate::macros::cast_to::cast_to!(usize, $e) {
            $n * $e
        } else {
            $crate::functions::lua_m_toobig::lua_m_toobig($l);
            usize::MAX
        }
    };
}

pub use lua_m_arraysize;

#[allow(unused_imports)]
pub use lua_m_arraysize as luaM_arraysize;
