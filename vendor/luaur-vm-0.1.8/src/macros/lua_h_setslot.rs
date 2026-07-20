//! Node: `cxx:Macro:Luau.VM:VM/src/ltable.h:35:luaH_setslot`
//! C++: `(invalidateTMcache(t), (slot == luaO_nilobject ? luaH_newkey(L, t, key) : cast_to(TValue*, slot)))`

#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaH_setslot {
    ($L:expr, $t:expr, $slot:expr, $key:expr) => {{
        $crate::macros::invalidate_t_mcache::invalidateTMcache($t);
        if $slot == $crate::macros::lua_o_nilobject::luaO_nilobject {
            $crate::functions::lua_h_newkey::lua_h_newkey($L, $t, $key)
        } else {
            $slot as *mut $crate::type_aliases::t_value::TValue
        }
    }};
}

pub use luaH_setslot;

#[allow(unused_imports)]
pub use luaH_setslot as lua_h_setslot;
