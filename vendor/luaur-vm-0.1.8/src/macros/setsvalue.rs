use crate::enums::lua_type::lua_Type;
use crate::macros::cast_to::cast_to;
use crate::macros::checkliveness::checkliveness;
use crate::records::gc_object::GCObject;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! setsvalue {
    ($L:expr, $obj:expr, $x:expr) => {{
        let i_o: *mut $crate::type_aliases::t_value::TValue = $obj;
        unsafe {
            (*i_o).value.gc =
                $crate::macros::cast_to::cast_to!(*mut $crate::records::gc_object::GCObject, $x);
            (*i_o).tt = $crate::enums::lua_type::lua_Type::LUA_TSTRING as core::ffi::c_int;
            $crate::macros::checkliveness::checkliveness!((*$L).global, i_o);
        }
    }};
}

pub use setsvalue;
