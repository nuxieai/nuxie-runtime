use crate::cast_int;
use crate::clvalue;
use crate::macros::nvalue::nvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::setobj_2_s::setobj_2_s;
use crate::macros::svalue::svalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::macros::ttisstring::ttisstring;
use crate::records::closure::Closure;
use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;
use luaur_common::FFlag;

#[allow(non_snake_case)]
pub unsafe fn luau_f_select(
    l: *mut LuaState,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    _args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams == 1 && nresults == 1 {
        let func = (*(*l).ci).func;
        let p = if FFlag::LuauCIProto.get() {
            (*(*l).ci).p
        } else {
            let cl = clvalue!(func);
            let lcl =
                core::ptr::addr_of!((*cl).inner.l).cast::<crate::records::closure::LClosure>();
            (*lcl).p
        };
        let n = cast_int!((*l).base.offset_from(func) as i32) - (*p).numparams as i32 - 1;

        if ttisnumber!(arg0) {
            let i = nvalue!(arg0) as i32;

            if ((i - 1) as u32) < (n as u32) {
                setobj_2_s!(l, res, (*l).base.offset((-n + (i - 1)) as isize));
                return 1;
            }
        } else if ttisstring!(arg0) && *svalue!(arg0) == b'#' as core::ffi::c_char {
            setnvalue!(res, n as f64);
            return 1;
        }
    }
    -1
}
