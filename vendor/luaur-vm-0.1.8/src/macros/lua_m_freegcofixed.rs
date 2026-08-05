#[allow(non_snake_case)]
#[macro_export]
macro_rules! lua_m_freegcofixed {
    ($L:expr, $p:expr, $size:expr, $memcat:expr, $page:expr) => {
        $crate::functions::lua_m_freegcofixed::luaM_freegcofixed_(
            $L,
            $p as *mut $crate::records::gc_object::GCObject,
            $size,
            $memcat,
            $page,
        )
    };
}

pub use lua_m_freegcofixed;
#[allow(unused_imports)]
pub use lua_m_freegcofixed as luaM_freegcofixed;
