use crate::enums::lua_type::lua_Type;
use crate::macros::lua_vector_size::LUA_VECTOR_SIZE;
use crate::macros::setvvalue::setvvalue;
use crate::macros::ttisvector::ttisvector;
use crate::macros::vvalue::vvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_vectornormalize(
    _L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    _args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 1 && nresults <= 1 && ttisvector!(arg0) {
        let v = vvalue!(arg0).as_ptr();

        if LUA_VECTOR_SIZE == 4 {
            let inv_sqrt = 1.0f32
                / ((*v.offset(0)) * (*v.offset(0))
                    + (*v.offset(1)) * (*v.offset(1))
                    + (*v.offset(2)) * (*v.offset(2))
                    + (*v.offset(3)) * (*v.offset(3)))
                .sqrt();

            setvvalue!(
                res,
                (*v.offset(0)) * inv_sqrt,
                (*v.offset(1)) * inv_sqrt,
                (*v.offset(2)) * inv_sqrt,
                (*v.offset(3)) * inv_sqrt
            );
        } else {
            let inv_sqrt = 1.0f32
                / ((*v.offset(0)) * (*v.offset(0))
                    + (*v.offset(1)) * (*v.offset(1))
                    + (*v.offset(2)) * (*v.offset(2)))
                .sqrt();

            setvvalue!(
                res,
                (*v.offset(0)) * inv_sqrt,
                (*v.offset(1)) * inv_sqrt,
                (*v.offset(2)) * inv_sqrt,
                0.0f32
            );
        }

        return 1;
    }

    -1
}
