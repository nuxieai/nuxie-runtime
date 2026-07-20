use crate::enums::lua_type::lua_Type;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::cast_num::cast_num;
use crate::macros::setnvalue::setnvalue;
use crate::type_aliases::lua_state::lua_State;

pub fn lua_pushinteger(l: *mut lua_State, n: core::ffi::c_int) {
    unsafe {
        setnvalue!((*l).top, cast_num!(n));
    }
    api_incr_top!(l);
}
