use crate::macros::cast_to::cast_to;
use crate::records::lua_node::LuaNode;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! gval2slot {
    ($t:expr, $v:expr) => {
        $crate::macros::cast_to::cast_to!(
            core::ffi::c_int,
            ($crate::macros::cast_to::cast_to!(
                *mut $crate::records::lua_node::LuaNode,
                $v as *const $crate::type_aliases::t_value::TValue
            ) as usize)
                .wrapping_sub((*$t).node as usize)
                / core::mem::size_of::<$crate::records::lua_node::LuaNode>()
        )
    };
}

pub use gval2slot;
