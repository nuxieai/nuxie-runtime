use crate::enums::lua_type::lua_Type;
use crate::functions::roundsd_sse_41::roundsd_sse41;
use crate::macros::luau_target_sse_41::LUAU_TARGET_SSE41;
use crate::macros::nvalue::nvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_round_sse_41(
    _L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    _args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if LUAU_TARGET_SSE41 && nparams >= 1 && nresults <= 1 && ttisnumber!(arg0) {
        let a1 = nvalue!(arg0);

        // roundsd only supports bankers rounding natively, so we need to emulate rounding by using truncation
        // offset is prevfloat(0.5), which is important so that we round prevfloat(0.5) to 0.
        const OFFSET: f64 = 0.49999999999999994;

        // _MM_FROUND_TO_ZERO is 3
        const MM_FROUND_TO_ZERO: i32 = 3;

        setnvalue!(
            res,
            roundsd_sse41::<MM_FROUND_TO_ZERO>(a1 + if a1 < 0.0 { -OFFSET } else { OFFSET })
        );

        1
    } else {
        -1
    }
}
