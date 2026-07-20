use crate::enums::lua_type::lua_Type;
use crate::macros::lua_vector_size::LUA_VECTOR_SIZE;
use crate::macros::nvalue::nvalue;
use crate::macros::setvvalue::setvvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_vector(
    _l: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 2 && nresults <= 1 && ttisnumber!(arg0) && ttisnumber!(args) {
        let x = nvalue!(arg0) as f32;
        let y = nvalue!(args) as f32;
        let z: f32 = if nparams >= 3 {
            if !ttisnumber!(args.add(1)) {
                return -1;
            }
            nvalue!(args.add(1)) as f32
        } else {
            0.0
        };

        if LUA_VECTOR_SIZE == 4 {
            let w: f32 = if nparams >= 4 {
                if !ttisnumber!(args.add(2)) {
                    return -1;
                }
                nvalue!(args.add(2)) as f32
            } else {
                0.0
            };

            setvvalue!(res, x, y, z, w);
        } else {
            setvvalue!(res, x, y, z, 0.0);
        }

        1
    } else {
        -1
    }
}
