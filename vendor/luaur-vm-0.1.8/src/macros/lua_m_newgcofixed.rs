#[allow(non_snake_case)]
#[macro_export]
macro_rules! lua_m_newgcofixed {
    ($L:expr, $t:ty, $size:expr, $memcat:expr) => {
        $crate::functions::lua_m_newgcofixed::luaM_newgcofixed_($L, $size, $memcat) as *mut $t
    };
}

pub use lua_m_newgcofixed;
#[allow(unused_imports)]
pub use lua_m_newgcofixed as luaM_newgcofixed;
