use crate::functions::lua_c_barrierback::lua_c_barrierback;
use crate::macros::isblack::isblack;
use crate::macros::obj_2_gco::obj2gco;
use crate::records::gc_object::GCObject;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaC_objectbarrier {
    ($l:expr) => {{
        let obj = $crate::macros::obj_2_gco::obj2gco!($l);
        if $crate::macros::isblack::isblack!(obj) {
            unsafe {
                $crate::functions::lua_c_barrierback::lua_c_barrierback(
                    core::ptr::null_mut(),
                    obj as *mut $crate::records::gc_object::GCObject,
                    &mut (*($l)).gclist,
                )
            }
        }
    }};
}

pub use luaC_objectbarrier;
