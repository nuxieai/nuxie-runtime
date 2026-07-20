use crate::functions::lua_m_newgco::luaM_newgco_;
use crate::macros::cast_to::cast_to;
use crate::records::gc_object::GCObject;
use crate::records::lua_state::lua_State;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! lua_m_newgco {
    ($L:expr, $t:ty, $size:expr, $memcat:expr) => {
        $crate::macros::cast_to::cast_to!(
            $t,
            $crate::functions::lua_m_newgco::luaM_newgco_(
                $L as *mut $crate::records::lua_state::lua_State,
                $size,
                $memcat
            ) as *mut $crate::records::gc_object::GCObject
        )
    };
}

pub use lua_m_newgco;

#[allow(unused_imports)]
pub use lua_m_newgco as luaM_newgco;
