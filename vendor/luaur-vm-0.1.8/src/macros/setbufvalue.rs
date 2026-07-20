#[allow(non_snake_case)]
#[macro_export]
macro_rules! setbufvalue {
    ($l:expr, $obj:expr, $x:expr) => {{
        let i_o = $obj;
        unsafe {
            (*i_o).value.gc = $crate::macros::cast_to::cast_to!(
                *mut $crate::type_aliases::gc_object::GcObject,
                $x
            );
            (*i_o).tt = $crate::enums::lua_type::lua_Type::LUA_TBUFFER as core::ffi::c_int;
            $crate::macros::checkliveness::checkliveness!((*$l).global, i_o);
        }
    }};
}

pub use setbufvalue;
