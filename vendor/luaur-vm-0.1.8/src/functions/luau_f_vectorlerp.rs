use crate::enums::lua_type::lua_Type;
use crate::functions::luai_lerpf::luai_lerpf;
use crate::macros::lua_vector_size::LUA_VECTOR_SIZE;
use crate::macros::setvvalue::setvvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::macros::ttisvector::ttisvector;
use crate::macros::vvalue::vvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_vectorlerp(
    _L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 3
        && nresults <= 1
        && ttisvector!(arg0)
        && ttisvector!(args)
        && ttisnumber!(args.offset(1))
    {
        let a = vvalue!(arg0).as_ptr();
        let b = vvalue!(args).as_ptr();
        let t = (crate::macros::nvalue::nvalue!(args.offset(1)));

        if LUA_VECTOR_SIZE == 4 {
            setvvalue!(
                res,
                luai_lerpf(a.offset(0).read(), b.offset(0).read(), t as f32),
                luai_lerpf(a.offset(1).read(), b.offset(1).read(), t as f32),
                luai_lerpf(a.offset(2).read(), b.offset(2).read(), t as f32),
                luai_lerpf(a.offset(3).read(), b.offset(3).read(), t as f32)
            );
        } else {
            setvvalue!(
                res,
                luai_lerpf(a.offset(0).read(), b.offset(0).read(), t as f32),
                luai_lerpf(a.offset(1).read(), b.offset(1).read(), t as f32),
                luai_lerpf(a.offset(2).read(), b.offset(2).read(), t as f32),
                0.0f32
            );
        }

        return 1;
    }

    -1
}
