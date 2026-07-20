use crate::enums::lua_type::lua_Type;
use crate::macros::cast_to::cast_to;
use crate::macros::checkliveness::checkliveness;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! setuvalue {
    ($L:expr, $obj:expr, $x:expr) => {{
        let i_o = $obj as *mut $crate::type_aliases::t_value::TValue;
        unsafe {
            (*i_o).value.gc =
                $crate::macros::cast_to::cast_to!($crate::type_aliases::gc_object::GCObject, $x);
            (*i_o).tt = $crate::enums::lua_type::lua_Type::LUA_TUSERDATA as core::ffi::c_int;
            $crate::macros::checkliveness::checkliveness!((*$L).global, i_o);
        }
    }};
}

pub use setuvalue;
