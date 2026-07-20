use crate::enums::lua_type::lua_Type;
use crate::macros::ttisvector::ttisvector;
use crate::macros::vvalue::vvalue;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luaV_tovector(obj: *const TValue) -> *const f32 {
    if ttisvector!(obj) {
        let v = vvalue!(obj);
        v.as_ptr()
    } else {
        core::ptr::null()
    }
}
