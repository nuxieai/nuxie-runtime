use crate::enums::lua_type::lua_Type;
use crate::macros::setnvalue::setnvalue;
use crate::macros::ttisvector::ttisvector;
use crate::macros::vvalue::vvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_vectorlengthsquared(
    _l: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    _args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 1 && nresults <= 1 && ttisvector!(arg0) {
        let v = vvalue!(arg0).as_ptr();
        setnvalue!(
            res,
            (v.add(0).read() * v.add(0).read()
                + v.add(1).read() * v.add(1).read()
                + v.add(2).read() * v.add(2).read()) as f64
        );
        return 1;
    }

    -1
}
