use crate::enums::lua_type::lua_Type;
use crate::macros::lua_vector_size::LUA_VECTOR_SIZE;
use crate::macros::setnvalue::setnvalue;
use crate::macros::ttisvector::ttisvector;
use crate::macros::vvalue::vvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_vectordot(
    _L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    let _ = lua_Type::LUA_TNIL;

    if nparams >= 2 && nresults <= 1 && ttisvector!(arg0) && ttisvector!(args) {
        let a = vvalue!(arg0).as_ptr();
        let b = vvalue!(args).as_ptr();

        if LUA_VECTOR_SIZE == 4 {
            setnvalue!(
                res,
                ((*a.offset(0)) * (*b.offset(0))
                    + (*a.offset(1)) * (*b.offset(1))
                    + (*a.offset(2)) * (*b.offset(2))
                    + (*a.offset(3)) * (*b.offset(3))) as f64
            );
        } else {
            setnvalue!(
                res,
                ((*a.offset(0)) * (*b.offset(0))
                    + (*a.offset(1)) * (*b.offset(1))
                    + (*a.offset(2)) * (*b.offset(2))) as f64
            );
        }

        return 1;
    }

    -1
}
