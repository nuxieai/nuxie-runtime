use crate::functions::lua_c_barrierback::lua_c_barrierback;
use crate::macros::isblack::isblack;
use crate::macros::obj_2_gco::obj2gco;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! lua_c_barrierfast {
    ($L:expr, $t:expr) => {
        if $crate::macros::isblack::isblack!($t as *mut $crate::records::gc_object::GCObject) {
            unsafe {
                $crate::functions::lua_c_barrierback::lua_c_barrierback(
                    $L as *mut $crate::records::lua_state::lua_State,
                    $t as *mut $crate::records::gc_object::GCObject,
                    &mut (*$t).gclist,
                );
            }
        }
    };
}

pub use lua_c_barrierfast;

#[allow(unused_imports)]
pub use lua_c_barrierfast as luaC_barrierfast;
