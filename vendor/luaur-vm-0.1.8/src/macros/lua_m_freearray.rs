use crate::functions::lua_m_free::luaM_free_;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! lua_m_freearray {
    ($L:expr, $b:expr, $n:expr, $t:ty, $memcat:expr) => {
        unsafe {
            $crate::functions::lua_m_free::luaM_free_(
                $L,
                $b as *mut core::ffi::c_void,
                ($n as usize) * core::mem::size_of::<$t>(),
                $memcat as u8,
            )
        }
    };
}

pub use lua_m_freearray;

#[allow(unused_imports)]
pub use lua_m_freearray as luaM_freearray;
