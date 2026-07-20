//! Node: `cxx:Macro:Luau.VM:VM/src/lmem.h:15:lua_m_newarray`
//! Source: `VM/src/lmem.h:15` (hand-fixed: was a `()` placeholder)

#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaM_newarray {
    ($L:expr, $n:expr, $t:ty, $memcat:expr) => {
        $crate::functions::lua_m_new::luaM_new_(
            $L,
            $crate::macros::lua_m_arraysize::luaM_arraysize!(
                $L,
                $n as usize,
                core::mem::size_of::<$t>()
            ),
            $memcat,
        ) as *mut $t
    };
}

pub use luaM_newarray;
