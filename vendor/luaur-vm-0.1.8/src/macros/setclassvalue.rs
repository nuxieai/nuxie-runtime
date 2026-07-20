use crate::enums::lua_type::lua_Type;
use crate::macros::cast_to::cast_to;
use crate::macros::checkliveness::checkliveness;
use crate::records::gc_object::GCObject;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! setclassvalue {
    ($L:expr, $obj:expr, $x:expr) => {
        unsafe {
            let i_o = $obj;
            (*i_o).value.gc =
                $crate::macros::cast_to::cast_to!(*mut $crate::records::gc_object::GCObject, $x);
            (*i_o).set_tt($crate::enums::lua_type::lua_Type::LUA_TCLASS as core::ffi::c_int);
            $crate::macros::checkliveness::checkliveness!((*$L).global, i_o);
        }
    };
}

pub use setclassvalue;
