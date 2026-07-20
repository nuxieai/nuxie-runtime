use crate::functions::lua_m_realloc::lua_m_realloc_;
use crate::macros::cast_to::cast_to;
use crate::macros::lua_m_arraysize::lua_m_arraysize;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! lua_m_reallocarray {
    ($L:expr, $v:expr, $oldn:expr, $n:expr, $t:ty, $memcat:expr) => {
        $v = $crate::macros::cast_to::cast_to!(
            *mut $t,
            $crate::functions::lua_m_realloc::lua_m_realloc_(
                $L,
                $v as *mut core::ffi::c_void,
                ($oldn) * core::mem::size_of::<$t>(),
                $crate::macros::lua_m_arraysize::lua_m_arraysize!(
                    $L,
                    $n,
                    core::mem::size_of::<$t>()
                ),
                $memcat as u8
            )
        )
    };
}

pub use lua_m_reallocarray;

#[allow(unused_imports)]
pub use lua_m_reallocarray as luaM_reallocarray;
