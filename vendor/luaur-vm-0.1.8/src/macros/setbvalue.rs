use crate::enums::lua_type::lua_Type;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! setbvalue {
    ($obj:expr, $x:expr) => {
        let i_o: *mut TValue = $obj;
        unsafe {
            (*i_o).value.b = $x as core::ffi::c_int;
            (*i_o).tt = $crate::enums::lua_type::lua_Type::LUA_TBOOLEAN as i32;
        }
    };
}

pub use setbvalue;
