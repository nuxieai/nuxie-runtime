use luaur_common::macros::luau_big_endian::LUAU_BIG_ENDIAN;

use crate::enums::lua_type::lua_Type;
use crate::macros::bufvalue::bufvalue;
use crate::macros::checkoutofbounds::checkoutofbounds;
use crate::macros::luai_num_2_int::luai_num2int;
use crate::macros::lvalue::lvalue;
use crate::macros::nvalue::nvalue;
use crate::macros::ttisbuffer::ttisbuffer;
use crate::macros::ttisinteger::ttisinteger;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_bufferwritelong(
    _l: *mut lua_State,
    _res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if !LUAU_BIG_ENDIAN
        && nparams >= 3
        && nresults <= 0
        && ttisbuffer!(arg0)
        && ttisnumber!(args)
        && ttisinteger!(args.wrapping_add(1))
    {
        let mut offset: core::ffi::c_int = 0;
        luai_num2int!(offset, nvalue!(args));

        let len = (*bufvalue!(arg0)).len as usize;
        if checkoutofbounds(offset, len, core::mem::size_of::<i64>()) {
            let val: i64 = lvalue!(args.wrapping_add(1));
            let dst = (*bufvalue!(arg0)).data.as_mut_ptr().add(offset as usize) as *mut i64;
            core::ptr::write_unaligned(dst, val);
            return 0;
        }
    }

    -1
}
