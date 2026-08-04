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
    if LUAU_TARGET_SSE41 {
        if luaur_common::FFlag::LuauMathRoundNegZero.get() {
            if nparams >= 1 && nresults <= 1 && ttisnumber!(arg0) {
                let a1 = nvalue!(arg0);
                // roundsd only supports bankers rounding natively, so we need to emulate rounding by using truncation
                // offset is prevfloat(0.5), which is important so that we round prevfloat(0.5) to 0.
                const OFFSET: f64 = 0.49999999999999994;

                #[cfg(target_arch = "x86_64")]
                {
                    use core::arch::x86_64::*;

                    let va1 = _mm_set_sd(a1);
                    let sign = _mm_and_pd(va1, _mm_set_sd(-0.0));
                    let off = _mm_or_pd(_mm_set_sd(OFFSET), sign);
                    let sum = _mm_add_sd(va1, off);
                    let result = _mm_round_sd(sum, sum, 3 | 8);
                    setnvalue!(res, _mm_cvtsd_f64(result));
                    return 1;
                }
            }
        } else if nparams >= 1 && nresults <= 1 && ttisnumber!(arg0) {
            let a1 = nvalue!(arg0);
            // roundsd only supports bankers rounding natively, so we need to emulate rounding by using truncation
            // offset is prevfloat(0.5), which is important so that we round prevfloat(0.5) to 0.
            const OFFSET: f64 = 0.49999999999999994;
            setnvalue!(
                res,
                roundsd_sse41::<3>(a1 + if a1 < 0.0 { -OFFSET } else { OFFSET })
            );
            return 1;
        }
    }

    -1
}
