use crate::enums::lua_type::lua_Type;
use crate::functions::lua_v_tonumber::lua_v_tonumber;
use crate::macros::ttype::ttype;

#[allow(non_snake_case)]
#[macro_export]
macro_rules! tonumber {
    ($o:expr, $n:expr) => {{
        if $crate::macros::ttype::ttype!($o)
            == ($crate::enums::lua_type::lua_Type::LUA_TNUMBER as i32)
        {
            true
        } else {
            let result = unsafe { $crate::functions::lua_v_tonumber::lua_v_tonumber($o, $n) };
            if !result.is_null() {
                $o = result as *mut _;
                true
            } else {
                false
            }
        }
    }};
}

pub use tonumber;
