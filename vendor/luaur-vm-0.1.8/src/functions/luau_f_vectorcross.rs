use crate::enums::lua_type::lua_Type;
use crate::macros::setvvalue::setvvalue;
use crate::macros::ttisvector::ttisvector;
use crate::macros::vvalue::vvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_vectorcross(
    _L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 2 && nresults <= 1 && ttisvector!(arg0) && ttisvector!(args) {
        let a = vvalue!(arg0).as_ptr();
        let b = vvalue!(args).as_ptr();

        // same for 3- and 4- wide vectors
        setvvalue!(
            res,
            a.offset(1).read() * b.offset(2).read() - a.offset(2).read() * b.offset(1).read(),
            a.offset(2).read() * b.offset(0).read() - a.offset(0).read() * b.offset(2).read(),
            a.offset(0).read() * b.offset(1).read() - a.offset(1).read() * b.offset(0).read(),
            0.0f32
        );
        return 1;
    }

    -1
}
