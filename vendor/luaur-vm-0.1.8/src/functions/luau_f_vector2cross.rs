use crate::enums::lua_type::lua_Type;
use crate::macros::setnvalue::setnvalue;
use crate::macros::ttisvector::ttisvector;
use crate::macros::vvalue::vvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_vector2cross(
    _l: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 2 && nresults <= 1 && ttisvector!(arg0) && ttisvector!(args) {
        let a = vvalue!(arg0).as_ptr();
        let b = vvalue!(args).as_ptr();
        setnvalue!(
            res,
            (a.add(0).read() * b.add(1).read() - a.add(1).read() * b.add(0).read()) as f64
        );
        return 1;
    }

    -1
}
